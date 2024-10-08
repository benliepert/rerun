use std::sync::{atomic::AtomicU64, OnceLock};

use ahash::HashMap;
use arrow2::{
    array::{Array as ArrowArray, DictionaryArray as ArrowDictionaryArray},
    chunk::Chunk as ArrowChunk,
    datatypes::Schema as ArrowSchema,
};
use itertools::Itertools;

use re_chunk::{Chunk, LatestAtQuery, RangeQuery, RowId, TimeInt};
use re_chunk_store::{ColumnDescriptor, ComponentColumnDescriptor, RangeQueryExpression};

use crate::{QueryEngine, RecordBatch};

// ---

/// A handle to a range query, ready to be executed.
///
/// Cheaply created via [`QueryEngine::range`].
///
/// See [`RangeQueryHandle::next_page`].
//
// TODO(cmc): pagination support
// TODO(cmc): intra-timestamp decimation support
pub struct RangeQueryHandle<'a> {
    /// Handle to the [`QueryEngine`].
    pub(crate) engine: &'a QueryEngine<'a>,

    /// The original query expression used to instantiate this handle.
    pub(crate) query: RangeQueryExpression,

    /// The user-specified schema that describes any data returned through this handle, if any.
    pub(crate) user_columns: Option<Vec<ColumnDescriptor>>,

    /// Internal private state. Lazily computed.
    ///
    /// It is important that handles stay cheap to create.
    state: OnceLock<RangeQuerytHandleState>,
}

/// Internal private state. Lazily computed.
struct RangeQuerytHandleState {
    /// The final schema.
    columns: Vec<ColumnDescriptor>,

    /// All the [`Chunk`]s for the active point-of-view.
    ///
    /// These are already sorted and vertically sliced according to the query.
    pov_chunks: Option<Vec<Chunk>>,

    /// Tracks the current page index. Used for [`RangeQueryHandle::next_page`].
    //
    // NOTE: The state is behind a `OnceLock`, the atomic just make some things simpler down the road.
    cur_page: AtomicU64,
}

impl<'a> RangeQueryHandle<'a> {
    pub(crate) fn new(
        engine: &'a QueryEngine<'a>,
        query: RangeQueryExpression,
        user_columns: Option<Vec<ColumnDescriptor>>,
    ) -> Self {
        Self {
            engine,
            query,
            user_columns,
            state: Default::default(),
        }
    }
}

impl RangeQueryHandle<'_> {
    /// Lazily initialize internal private state.
    ///
    /// It is important that handles stay cheap to create.
    fn init(&self) -> &RangeQuerytHandleState {
        self.state.get_or_init(|| {
            re_tracing::profile_scope!("init");

            let columns = {
                re_tracing::profile_scope!("compute schema");

                self.user_columns.clone().unwrap_or_else(|| {
                    self.engine
                        .store
                        .schema_for_query(&self.query.clone().into())
                })
            };

            let pov_chunks = {
                re_tracing::profile_scope!("gather pov timestamps");

                let query = RangeQuery::new(self.query.timeline, self.query.time_range)
                    .keep_extra_timelines(true) // we want all the timelines we can get!
                    .keep_extra_components(false);

                let results = self.engine.cache.range(
                    self.engine.store,
                    &query,
                    &self.query.pov.entity_path,
                    [self.query.pov.component_name],
                );

                results
                    .components
                    .into_iter()
                    .find_map(|(component_name, chunks)| {
                        (component_name == self.query.pov.component_name).then_some(chunks)
                    })
            };

            RangeQuerytHandleState {
                columns,
                pov_chunks,
                cur_page: AtomicU64::new(0),
            }
        })
    }

    /// All results returned by this handle will strictly follow this schema.
    ///
    /// Columns that do not yield any data will still be present in the results, filled with null values.
    pub fn schema(&self) -> &[ColumnDescriptor] {
        &self.init().columns
    }

    /// Partially executes the range query until the next natural page of results.
    ///
    /// Returns a single [`RecordBatch`] containing as many rows as available in the page, or
    /// `None` if all the dataset has been returned.
    /// Each cell in the result corresponds to the latest known value at that particular point in time
    /// for each respective `ColumnDescriptor`.
    ///
    /// The schema of the returned [`RecordBatch`] is guaranteed to match the one returned by
    /// [`Self::schema`].
    /// Columns that do not yield any data will still be present in the results, filled with null values.
    ///
    /// "Natural pages" refers to pages of data that match 1:1 to the underlying storage.
    /// The size of each page cannot be known in advance, as it depends on unspecified
    /// implementation details.
    /// This is the most performant way to iterate over the dataset.
    ///
    /// ```ignore
    /// while let Some(batch) = query_handle.next_page() {
    ///     // …
    /// }
    /// ```
    pub fn next_page(&mut self) -> Option<RecordBatch> {
        re_tracing::profile_function!(format!("{:?}", self.query));

        let state = self.init();
        let cur_page = state.cur_page.load(std::sync::atomic::Ordering::Relaxed);
        let pov_chunk = state.pov_chunks.as_ref()?.get(cur_page as usize)?;
        _ = state
            .cur_page
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.dense_batch_at_pov(pov_chunk)
    }

    /// Partially executes the range query in order to return the specified range of rows.
    ///
    /// Returns a vector of [`RecordBatch`]es: as many as required to fill the specified range.
    /// Each [`RecordBatch`] will correspond a "natural page" of data, even the first and last batch,
    /// although they might be cut off at the edge.
    /// Each cell in the result corresponds to the latest known value at that particular point in time
    /// for each respective `ColumnDescriptor`.
    ///
    /// The schema of the returned [`RecordBatch`]es is guaranteed to match the one returned by
    /// [`Self::schema`].
    /// Columns that do not yield any data will still be present in the results, filled with null values.
    ///
    /// "Natural pages" refers to pages of data that match 1:1 to the underlying storage.
    /// The size of each page cannot be known in advance, as it depends on unspecified
    /// implementation details.
    /// This is the most performant way to iterate over the dataset.
    //
    // TODO(cmc): This could be turned into an actual lazy iterator at some point.
    pub fn get(&self, offset: u64, mut len: u64) -> Vec<RecordBatch> {
        let mut results = Vec::new();

        let state = self.init();
        let Some(pov_chunks) = state.pov_chunks.as_ref() else {
            return results;
        };
        let mut pov_chunks = pov_chunks.iter();

        let mut cur_offset = 0;
        let Some(mut cur_pov_chunk) = pov_chunks.next().cloned() else {
            return results;
        };

        // Fast-forward until the first relevant PoV chunk.
        //
        // TODO(cmc): should keep an extra sorted datastructure and use a binsearch instead.
        while (cur_offset + cur_pov_chunk.num_rows() as u64) < offset {
            cur_offset += cur_pov_chunk.num_rows() as u64;

            let Some(next_pov_chunk) = pov_chunks.next().cloned() else {
                return results;
            };
            cur_pov_chunk = next_pov_chunk;
        }

        // Fast-forward to until the first relevant row in the PoV chunk.
        let mut offset = if cur_offset < offset {
            offset.saturating_sub(cur_offset)
        } else {
            0
        };

        // Repeatedly compute dense ranges until we've returned `len` rows.
        while len > 0 {
            cur_pov_chunk = cur_pov_chunk.row_sliced(offset as _, len as _);
            results.extend(self.dense_batch_at_pov(&cur_pov_chunk));

            offset = 0; // always start at the first row after the first chunk
            len = len.saturating_sub(cur_pov_chunk.num_rows() as u64);

            let Some(next_pov_chunk) = pov_chunks.next().cloned() else {
                break;
            };
            cur_pov_chunk = next_pov_chunk;
        }

        results
    }

    /// How many chunks / natural pages of data will be returned?
    #[inline]
    pub fn num_chunks(&self) -> u64 {
        self.init()
            .pov_chunks
            .as_ref()
            .map_or(0, |pov_chunks| pov_chunks.len() as _)
    }

    /// How many rows of data will be returned?
    #[inline]
    pub fn num_rows(&self) -> u64 {
        self.init().pov_chunks.as_ref().map_or(0, |pov_chunks| {
            pov_chunks.iter().map(|chunk| chunk.num_rows() as u64).sum()
        })
    }

    fn dense_batch_at_pov(&self, pov_chunk: &Chunk) -> Option<RecordBatch> {
        let pov_time_column = pov_chunk.timelines().get(&self.query.timeline)?;
        let columns = self.schema();

        // TODO(cmc): There are more efficient, albeit infinitely more complicated ways to do this.
        // Let's first implement all features (multi-PoV, pagination, timestamp streaming, etc) and
        // see if this ever becomes an issue before going down this road.
        //
        // TODO(cmc): Opportunities for parallelization, if it proves to be a net positive in practice.
        let dict_arrays: HashMap<&ComponentColumnDescriptor, ArrowDictionaryArray<u32>> = {
            re_tracing::profile_scope!("queries");

            columns
                .iter()
                .filter_map(|descr| match descr {
                    ColumnDescriptor::Component(descr) => Some(descr),
                    _ => None,
                })
                .filter_map(|descr| {
                    let arrays = pov_time_column
                        .times()
                        .map(|time| {
                            let query = LatestAtQuery::new(self.query.timeline, time);

                            let results = self.engine.cache.latest_at(
                                self.engine.store,
                                &query,
                                &descr.entity_path,
                                [descr.component_name],
                            );

                            results
                                .components
                                .get(&descr.component_name)
                                .and_then(|unit| {
                                    unit.component_batch_raw(&descr.component_name).clone().map(
                                        |array| {
                                            (
                                                unit.index(&query.timeline())
                                                    // NOTE: technically cannot happen, but better than unwrapping.
                                                    .unwrap_or((TimeInt::STATIC, RowId::ZERO)),
                                                array,
                                            )
                                        },
                                    )
                                })
                        })
                        .collect_vec();
                    let arrays = arrays
                        .iter()
                        .map(|array| {
                            array
                                .as_ref()
                                .map(|(index, array)| (index, &**array as &dyn ArrowArray))
                        })
                        .collect_vec();

                    let dict_array = {
                        re_tracing::profile_scope!("concat");
                        re_chunk::util::arrays_to_dictionary(descr.datatype.clone(), &arrays)
                    };

                    if cfg!(debug_assertions) {
                        #[allow(clippy::unwrap_used)] // want to crash in dev
                        Some((descr, dict_array.unwrap()))
                    } else {
                        // NOTE: Technically cannot ever happen, but I'd rather that than an uwnrap.
                        dict_array.map(|dict_array| (descr, dict_array))
                    }
                })
                .collect()
        };

        // NOTE: Keep in mind this must match the ordering specified by `Self::schema`.
        let packed_arrays = {
            re_tracing::profile_scope!("packing");

            columns
                .iter()
                .map(|descr| match descr {
                    ColumnDescriptor::Control(_descr) => pov_chunk.row_ids_array().to_boxed(),

                    ColumnDescriptor::Time(descr) => {
                        let time_column = pov_chunk.timelines().get(&descr.timeline).cloned();
                        time_column.map_or_else(
                            || {
                                arrow2::array::new_null_array(
                                    descr.datatype.clone(),
                                    pov_chunk.num_rows(),
                                )
                            },
                            |time_column| time_column.times_array().to_boxed(),
                        )
                    }

                    ColumnDescriptor::Component(descr) => dict_arrays.get(descr).map_or_else(
                        || {
                            arrow2::array::new_null_array(
                                descr.datatype.clone(),
                                pov_time_column.num_rows(),
                            )
                        },
                        |dict_array| dict_array.to_boxed(),
                    ),
                })
                .collect_vec()
        };

        Some(RecordBatch {
            schema: ArrowSchema {
                fields: columns
                    .iter()
                    .zip(packed_arrays.iter())
                    .map(|(descr, arr)| descr.to_arrow_field(Some(arr.data_type().clone())))
                    .collect(),
                metadata: Default::default(),
            },
            data: ArrowChunk::new(packed_arrays),
        })
    }
}

impl<'a> RangeQueryHandle<'a> {
    #[allow(clippy::should_implement_trait)] // we need an anonymous closure, this won't work
    pub fn into_iter(mut self) -> impl Iterator<Item = RecordBatch> + 'a {
        std::iter::from_fn(move || self.next_page())
    }
}

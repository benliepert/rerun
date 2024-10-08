# DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/python/mod.rs
# Based on "crates/store/re_types/definitions/rerun/archetypes/video.fbs".

# You can extend this class by creating a "AssetVideoExt" class in "asset_video_ext.py".

from __future__ import annotations

from attrs import define, field

from .. import components
from .._baseclasses import (
    Archetype,
)
from .asset_video_ext import AssetVideoExt

__all__ = ["AssetVideo"]


@define(str=False, repr=False, init=False)
class AssetVideo(AssetVideoExt, Archetype):
    """
    **Archetype**: A video file.

    NOTE: Videos can only be viewed in the Rerun web viewer.
    Only MP4 and AV1 is currently supported, and not in all browsers.
    Follow <https://github.com/rerun-io/rerun/issues/7298> for updates on the native support.

    ⚠️ **This is an experimental API! It is not fully supported, and is likely to change significantly in future versions.**
    """

    # __init__ can be found in asset_video_ext.py

    def __attrs_clear__(self) -> None:
        """Convenience method for calling `__attrs_init__` with all `None`s."""
        self.__attrs_init__(
            blob=None,  # type: ignore[arg-type]
            media_type=None,  # type: ignore[arg-type]
        )

    @classmethod
    def _clear(cls) -> AssetVideo:
        """Produce an empty AssetVideo, bypassing `__init__`."""
        inst = cls.__new__(cls)
        inst.__attrs_clear__()
        return inst

    blob: components.BlobBatch = field(
        metadata={"component": "required"},
        converter=components.BlobBatch._required,  # type: ignore[misc]
    )
    # The asset's bytes.
    #
    # (Docstring intentionally commented out to hide this field from the docs)

    media_type: components.MediaTypeBatch | None = field(
        metadata={"component": "optional"},
        default=None,
        converter=components.MediaTypeBatch._optional,  # type: ignore[misc]
    )
    # The Media Type of the asset.
    #
    # Supported values:
    # * `video/mp4`
    #
    # If omitted, the viewer will try to guess from the data blob.
    # If it cannot guess, it won't be able to render the asset.
    #
    # (Docstring intentionally commented out to hide this field from the docs)

    __str__ = Archetype.__str__
    __repr__ = Archetype.__repr__  # type: ignore[assignment]

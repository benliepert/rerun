// DO NOT EDIT! This file was auto-generated by crates/re_types_builder/src/codegen/cpp/mod.rs
// Based on "crates/re_types/definitions/rerun/blueprint/components/query_expressions.fbs".

#pragma once

#include "../../blueprint/datatypes/query_expressions.hpp"
#include "../../result.hpp"

#include <cstdint>
#include <memory>
#include <utility>

namespace arrow {
    class Array;
    class DataType;
    class StructBuilder;
} // namespace arrow

namespace rerun::blueprint::components {
    /// **Component**: A set of expressions used for a `DataQueryBlueprint`.
    ///
    /// Unstable. Used for the ongoing blueprint experimentations.
    struct QueryExpressions {
        rerun::blueprint::datatypes::QueryExpressions expressions;

      public:
        QueryExpressions() = default;

        QueryExpressions(rerun::blueprint::datatypes::QueryExpressions expressions_)
            : expressions(std::move(expressions_)) {}

        QueryExpressions& operator=(rerun::blueprint::datatypes::QueryExpressions expressions_) {
            expressions = std::move(expressions_);
            return *this;
        }

        /// Cast to the underlying QueryExpressions datatype
        operator rerun::blueprint::datatypes::QueryExpressions() const {
            return expressions;
        }
    };
} // namespace rerun::blueprint::components

namespace rerun {
    template <typename T>
    struct Loggable;

    /// \private
    template <>
    struct Loggable<blueprint::components::QueryExpressions> {
        static constexpr const char Name[] = "rerun.blueprint.components.QueryExpressions";

        /// Returns the arrow data type this type corresponds to.
        static const std::shared_ptr<arrow::DataType>& arrow_datatype();

        /// Fills an arrow array builder with an array of this type.
        static rerun::Error fill_arrow_array_builder(
            arrow::StructBuilder* builder, const blueprint::components::QueryExpressions* elements,
            size_t num_elements
        );

        /// Serializes an array of `rerun::blueprint:: components::QueryExpressions` into an arrow array.
        static Result<std::shared_ptr<arrow::Array>> to_arrow(
            const blueprint::components::QueryExpressions* instances, size_t num_instances
        );
    };
} // namespace rerun

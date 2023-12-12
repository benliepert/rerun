# DO NOT EDIT! This file was auto-generated by crates/re_types_builder/src/codegen/python.rs
# Based on "crates/re_types/definitions/rerun/blueprint/components/space_view_component.fbs".

# You can extend this class by creating a "SpaceViewComponentExt" class in "space_view_component_ext.py".

from __future__ import annotations

from ..._baseclasses import ComponentBatchMixin
from .. import datatypes

__all__ = ["SpaceViewComponent", "SpaceViewComponentBatch", "SpaceViewComponentType"]


class SpaceViewComponent(datatypes.SpaceViewComponent):
    """
    **Component**: A view of a space.

    Unstable. Used for the ongoing blueprint experimentations.
    """

    # You can define your own __init__ function as a member of SpaceViewComponentExt in space_view_component_ext.py

    # Note: there are no fields here because SpaceViewComponent delegates to datatypes.SpaceViewComponent
    pass


class SpaceViewComponentType(datatypes.SpaceViewComponentType):
    _TYPE_NAME: str = "rerun.blueprint.components.SpaceViewComponent"


class SpaceViewComponentBatch(datatypes.SpaceViewComponentBatch, ComponentBatchMixin):
    _ARROW_TYPE = SpaceViewComponentType()

# DO NOT EDIT! This file was auto-generated by crates/build/re_types_builder/src/codegen/python/mod.rs
# Based on "crates/store/re_types/definitions/rerun/archetypes/segmentation_image.fbs".

# You can extend this class by creating a "SegmentationImageExt" class in "segmentation_image_ext.py".

from __future__ import annotations

from attrs import define, field

from .. import components
from .._baseclasses import (
    Archetype,
)
from .segmentation_image_ext import SegmentationImageExt

__all__ = ["SegmentationImage"]


@define(str=False, repr=False, init=False)
class SegmentationImage(SegmentationImageExt, Archetype):
    """
    **Archetype**: An image made up of integer [`components.ClassId`][rerun.components.ClassId]s.

    Each pixel corresponds to a [`components.ClassId`][rerun.components.ClassId] that will be mapped to a color based on annotation context.

    In the case of floating point images, the label will be looked up based on rounding to the nearest
    integer value.

    See also [`archetypes.AnnotationContext`][rerun.archetypes.AnnotationContext] to associate each class with a color and a label.

    Example
    -------
    ### Simple segmentation image:
    ```python
    import numpy as np
    import rerun as rr

    # Create a segmentation image
    image = np.zeros((8, 12), dtype=np.uint8)
    image[0:4, 0:6] = 1
    image[4:8, 6:12] = 2

    rr.init("rerun_example_segmentation_image", spawn=True)

    # Assign a label and color to each class
    rr.log("/", rr.AnnotationContext([(1, "red", (255, 0, 0)), (2, "green", (0, 255, 0))]), static=True)

    rr.log("image", rr.SegmentationImage(image))
    ```
    <center>
    <picture>
      <source media="(max-width: 480px)" srcset="https://static.rerun.io/segmentation_image_simple/eb49e0b8cb870c75a69e2a47a2d202e5353115f6/480w.png">
      <source media="(max-width: 768px)" srcset="https://static.rerun.io/segmentation_image_simple/eb49e0b8cb870c75a69e2a47a2d202e5353115f6/768w.png">
      <source media="(max-width: 1024px)" srcset="https://static.rerun.io/segmentation_image_simple/eb49e0b8cb870c75a69e2a47a2d202e5353115f6/1024w.png">
      <source media="(max-width: 1200px)" srcset="https://static.rerun.io/segmentation_image_simple/eb49e0b8cb870c75a69e2a47a2d202e5353115f6/1200w.png">
      <img src="https://static.rerun.io/segmentation_image_simple/eb49e0b8cb870c75a69e2a47a2d202e5353115f6/full.png" width="640">
    </picture>
    </center>

    """

    # __init__ can be found in segmentation_image_ext.py

    def __attrs_clear__(self) -> None:
        """Convenience method for calling `__attrs_init__` with all `None`s."""
        self.__attrs_init__(
            data=None,  # type: ignore[arg-type]
            format=None,  # type: ignore[arg-type]
            opacity=None,  # type: ignore[arg-type]
            draw_order=None,  # type: ignore[arg-type]
        )

    @classmethod
    def _clear(cls) -> SegmentationImage:
        """Produce an empty SegmentationImage, bypassing `__init__`."""
        inst = cls.__new__(cls)
        inst.__attrs_clear__()
        return inst

    data: components.BlobBatch = field(
        metadata={"component": "required"},
        converter=components.BlobBatch._required,  # type: ignore[misc]
    )
    # The raw image data.
    #
    # (Docstring intentionally commented out to hide this field from the docs)

    format: components.ImageFormatBatch = field(
        metadata={"component": "required"},
        converter=components.ImageFormatBatch._required,  # type: ignore[misc]
    )
    # The format of the image.
    #
    # (Docstring intentionally commented out to hide this field from the docs)

    opacity: components.OpacityBatch | None = field(
        metadata={"component": "optional"},
        default=None,
        converter=components.OpacityBatch._optional,  # type: ignore[misc]
    )
    # Opacity of the image, useful for layering the segmentation image on top of another image.
    #
    # Defaults to 0.5 if there's any other images in the scene, otherwise 1.0.
    #
    # (Docstring intentionally commented out to hide this field from the docs)

    draw_order: components.DrawOrderBatch | None = field(
        metadata={"component": "optional"},
        default=None,
        converter=components.DrawOrderBatch._optional,  # type: ignore[misc]
    )
    # An optional floating point value that specifies the 2D drawing order.
    #
    # Objects with higher values are drawn on top of those with lower values.
    #
    # (Docstring intentionally commented out to hide this field from the docs)

    __str__ = Archetype.__str__
    __repr__ = Archetype.__repr__  # type: ignore[assignment]

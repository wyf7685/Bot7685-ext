# ruff: noqa: PYI021

from collections.abc import Callable, Mapping
from types import GenericAlias
from typing import Any, Final, overload

from bot7685_ext.wplace.consts import ColorName

__version__: Final[str]
__build_time__: Final[str]
__git_commit_hash__: Final[str]

class __WplaceModule:
    COLORS_MAP: Final[list[tuple[ColorName, tuple[int, int, int]]]]

    @staticmethod
    def template_compare(
        template_bytes: bytes,
        actual_bytes: bytes,
        include_pixels: bool = False,
        /,
    ) -> list[tuple[ColorName, int, int, list[tuple[int, int]]]]: ...
    @staticmethod
    def template_overlay(
        template_bytes: bytes,
        actual_bytes: bytes,
        overlay_alpha: int,
        /,
    ) -> bytes: ...
    @staticmethod
    def group_adjacent(
        points: list[tuple[int, int, int]],
        min_group_size: int,
        merge_distance: float,
        /,
    ) -> list[list[tuple[int, int, int]]]: ...
    @staticmethod
    def compose_tiles(
        tiles: list[tuple[tuple[int, int], bytes]],
        coord1: tuple[int, int, int, int],
        coord2: tuple[int, int, int, int],
        background: tuple[int, int, int] | None,
        /,
    ) -> bytes: ...

wplace: Final[__WplaceModule]

class LRU[KT, VT]:
    """LRU dict that can store up to ``size`` elements.

    An LRU dict behaves like a standard dict, except that it stores only fixed
    set of elements. Once the size overflows, it evicts least recently used
    items. If a callback is set it will call the callback with the evicted key
    and item.
    """

    @overload
    def __init__(
        self,
        size: int,
        callback: None = None,
    ) -> None: ...
    @overload
    def __init__(
        self,
        size: int,
        callback: Callable[[KT, VT], Any],
    ) -> None: ...
    def __class_getitem__(cls, item: Any, /) -> GenericAlias: ...
    def __len__(self) -> int: ...
    def __contains__(self, key: KT, /) -> bool: ...
    def __getitem__(self, key: KT, /) -> VT: ...
    def __setitem__(self, key: KT, value: VT, /) -> None: ...
    def __delitem__(self, key: KT, /) -> None: ...
    @overload
    def get(self, key: KT) -> VT | None:
        """If L has key return its value, otherwise None."""
    @overload
    def get[T](self, key: KT, instead: T) -> VT | T:
        """If L has key return its value, otherwise instead."""
    @overload
    def setdefault[T](self: LRU[KT, T | None], key: KT) -> VT | None:
        """If L has key return its value, otherwise insert key with a value of
        None and return None."""
    @overload
    def setdefault[T](self, key: KT, default: T) -> VT | T:
        """If L has key return its value, otherwise insert key with a value of
        default and return default."""
    @overload
    def pop(self, key: KT) -> VT:
        """If L has key return its value and remove it from L, otherwise raise
        KeyError."""
    @overload
    def pop[T](self, key: KT, default: VT | T) -> VT | T:
        """If L has key return its value and remove it from L, otherwise return
        default."""
    def popitem(self, least_recent: bool = True) -> tuple[KT, VT]:
        """Returns and removes a (key, value) pair. The pair returned is the
        least-recently used if least_recent is true, or the most-recently used
        if false."""
    def keys(self) -> list[KT]:
        """List of L's keys in MRU order."""
    def values(self) -> list[VT]:
        """List of L's values in MRU order."""
    def items(self) -> list[tuple[KT, VT]]:
        """List of L's items (key, value) in MRU order."""
    def has_key(self, key: KT) -> bool:
        """Check if key is there in L."""
    def set_size(self, size: int) -> None:
        """Set size of LRU."""
    def get_size(self) -> int:
        """Get size of LRU."""
    def clear(self) -> None:
        """Clear LRU."""
    def get_stats(self) -> tuple[int, int]:
        """Returns a tuple with cache hits and misses."""
    def peek_first_item(self) -> tuple[KT, VT] | None:
        """Returns the MRU item (key, value) without changing key order."""
    def peek_last_item(self) -> tuple[KT, VT] | None:
        """Returns the LRU item (key, value) without changing key order."""
    @overload
    def update(self, mapping: Mapping[KT, VT], /) -> None:
        """Update value for key in LRU."""
    @overload
    def update(self: LRU[str, VT], **kwargs: VT) -> None:
        """Update value for key in LRU."""
    @overload
    def update(self: LRU[str, VT], mapping: Mapping[str, VT], /, **kwargs: VT) -> None:
        """Update value for key in LRU."""
    @overload
    def set_callback(self, callback: Callable[[KT, VT], Any]) -> None:
        """Set a callback to call when an item is evicted."""
    @overload
    def set_callback(self, callback: None) -> None:
        """Unset the eviction callback."""

from asyncio import AbstractEventLoop
from collections.abc import Mapping
from types import GenericAlias
from typing import Any, Callable, Generic, TypeVar, overload

from bot7685_ext.wplace.consts import ColorName

_KT = TypeVar("_KT")
_VT = TypeVar("_VT")
_T = TypeVar("_T")

__version__: str
__build_time__: str
__git_commit_hash__: str

WPLACE_COLORS_MAP: list[tuple[ColorName, tuple[int, int, int]]]

async def wplace_template_compare(
    template_bytes: bytes,
    actual_bytes: bytes,
    include_pixels: bool,
    asyncio_loop: AbstractEventLoop,
    /,
) -> list[tuple[ColorName, int, int, list[tuple[int, int]]]]: ...
async def wplace_template_overlay(
    template_bytes: bytes,
    actual_bytes: bytes,
    overlay_alpha: int,
    asyncio_loop: AbstractEventLoop,
    /,
) -> bytes: ...
async def wplace_group_adjacent(
    points: list[tuple[int, int, int]],
    min_group_size: int,
    merge_distance: float,
    asyncio_loop: AbstractEventLoop,
    /,
) -> list[list[tuple[int, int, int]]]: ...
async def wplace_compose_tiles(
    tiles: list[tuple[tuple[int, int], bytes]],
    coord1: tuple[int, int, int, int],
    coord2: tuple[int, int, int, int],
    background: tuple[int, int, int] | None,
    asyncio_loop: AbstractEventLoop,
    /,
) -> bytes: ...

class LRU(Generic[_KT, _VT]):
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
        callback: Callable[[_KT, _VT], Any],
    ) -> None: ...
    def __class_getitem__(cls, item: Any, /) -> GenericAlias: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...
    def __contains__(self, key: _KT, /) -> bool: ...
    def __getitem__(self, key: _KT, /) -> _VT: ...
    def __setitem__(self, key: _KT, value: _VT, /) -> None: ...
    def __delitem__(self, key: _KT, /) -> None: ...
    @overload
    def get(self, key: _KT) -> _VT | None:
        """If L has key return its value, otherwise None."""
        ...
    @overload
    def get(self, key: _KT, instead: _T) -> _VT | _T:
        """If L has key return its value, otherwise instead."""
        ...
    @overload
    def setdefault(self: LRU[_KT, _T | None], key: _KT) -> _VT | None:
        """If L has key return its value, otherwise insert key with a value of
        None and return None."""
        ...
    @overload
    def setdefault(self, key: _KT, default: _T) -> _VT | _T:
        """If L has key return its value, otherwise insert key with a value of
        default and return default."""
        ...
    @overload
    def pop(self, key: _KT) -> _VT:
        """If L has key return its value and remove it from L, otherwise raise KeyError."""
        ...
    @overload
    def pop(self, key: _KT, default: _VT | _T) -> _VT | _T:
        """If L has key return its value and remove it from L, otherwise return default."""
        ...
    def popitem(self, least_recent: bool = True) -> tuple[_KT, _VT]:
        """Returns and removes a (key, value) pair. The pair returned is the
        least-recently used if least_recent is true, or the most-recently used if false."""
        ...
    def keys(self) -> list[_KT]:
        """List of L's keys in MRU order."""
        ...
    def values(self) -> list[_VT]:
        """List of L's values in MRU order."""
        ...
    def items(self) -> list[tuple[_KT, _VT]]:
        """List of L's items (key, value) in MRU order."""
        ...
    def has_key(self, key: _KT) -> bool:
        """Check if key is there in L."""
        ...
    def set_size(self, size: int) -> None:
        """Set size of LRU."""
        ...
    def get_size(self) -> int:
        """Get size of LRU."""
        ...
    def clear(self) -> None:
        """Clear LRU."""
        ...
    def get_stats(self) -> tuple[int, int]:
        """Returns a tuple with cache hits and misses."""
        ...
    def peek_first_item(self) -> tuple[_KT, _VT] | None:
        """Returns the MRU item (key, value) without changing key order."""
        ...
    def peek_last_item(self) -> tuple[_KT, _VT] | None:
        """Returns the LRU item (key, value) without changing key order."""
        ...
    @overload
    def update(self, mapping: Mapping[_KT, _VT], /) -> None:
        """Update value for key in LRU."""
        ...
    @overload
    def update(self: LRU[_KT | str, _VT], **kwargs: _VT) -> None:
        """Update value for key in LRU."""
        ...
    @overload
    def update(
        self: LRU[_KT | str, _VT], mapping: Mapping[str, _VT], /, **kwargs: _VT
    ) -> None:
        """Update value for key in LRU."""
        ...
    @overload
    def set_callback(self, callback: Callable[[_KT, _VT], Any]) -> None:
        """Set a callback to call when an item is evicted."""
        ...
    @overload
    def set_callback(self, callback: None) -> None:
        """Unset the eviction callback."""
        ...

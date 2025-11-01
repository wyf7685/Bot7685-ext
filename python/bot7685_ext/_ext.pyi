from asyncio import AbstractEventLoop

from bot7685_ext.wplace.consts import ColorName

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

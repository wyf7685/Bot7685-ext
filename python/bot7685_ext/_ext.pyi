from asyncio import AbstractEventLoop

__version__: str
__build_time__: str
__git_commit_hash__: str

async def wplace_template_compare(
    template_bytes: bytes,
    actual_bytes: bytes,
    include_pixels: bool,
    asyncio_loop: AbstractEventLoop,
    /,
) -> list[tuple[str, int, int, list[tuple[int, int]]]]: ...
async def wplace_template_overlay(
    template_bytes: bytes,
    actual_bytes: bytes,
    overlay_alpha: int,
    asyncio_loop: AbstractEventLoop,
    /,
) -> bytes: ...

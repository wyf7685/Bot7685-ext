from asyncio import AbstractEventLoop

async def wplace_template_compare(
    template_bytes: bytes,
    actual_bytes: bytes,
    include_pixels: bool,
    asyncio_loop: AbstractEventLoop,
    /,
) -> list[tuple[str, int, int, list[tuple[int, int]]]]: ...

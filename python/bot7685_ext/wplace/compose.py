import asyncio

from .._ext import wplace_compose_tiles


async def compose_tiles(
    tiles: list[tuple[tuple[int, int], bytes]],
    coord1: tuple[int, int, int, int],
    coord2: tuple[int, int, int, int],
    background: tuple[int, int, int] | None = None,
) -> bytes:
    return await wplace_compose_tiles(
        tiles, coord1, coord2, background, asyncio.get_event_loop()
    )

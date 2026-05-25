import asyncio

from .._ext import wplace_group_adjacent


async def group_adjacent(
    points: list[tuple[int, int, int]],
    min_group_size: int = 100,
    merge_distance: int | float = 30.0,
) -> list[list[tuple[int, int, int]]]:
    return await asyncio.to_thread(
        wplace_group_adjacent,
        points,
        int(min_group_size),
        float(merge_distance),
    )

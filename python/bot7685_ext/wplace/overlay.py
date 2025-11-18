import asyncio

from .._ext import wplace_template_overlay


async def overlay(
    template_bytes: bytes,
    actual_bytes: bytes,
    overlay_alpha: int = 96,
) -> bytes:
    return await wplace_template_overlay(
        template_bytes,
        actual_bytes,
        overlay_alpha,
        asyncio.get_event_loop(),
    )

import asyncio
from dataclasses import dataclass, field

from .._ext import wplace_template_compare
from .consts import ALL_COLORS, COLORS_ID, PAID_COLORS, ColorName


@dataclass
class ColorEntry:
    name: ColorName
    count: int = 0
    total: int = 0
    pixels: list[tuple[int, int]] = field(default_factory=list)

    @property
    def is_paid(self) -> bool:
        return self.name in PAID_COLORS

    @property
    def rgb(self) -> tuple[int, int, int]:
        return ALL_COLORS[self.name]

    @property
    def rgb_str(self) -> str:
        if self.name == "Transparent":
            return "transparent"

        r, g, b = self.rgb
        return f"#{r:02X}{g:02X}{b:02X}"

    @property
    def id(self) -> int:
        return COLORS_ID[self.name]

    @property
    def drawn(self) -> int:
        return self.total - self.count

    @property
    def progress(self) -> float:
        return (self.drawn / self.total * 100) if self.total > 0 else 0


async def compare(
    template_bytes: bytes,
    actual_bytes: bytes,
    include_pixels: bool = False,
) -> list[ColorEntry]:
    entries = await wplace_template_compare(
        template_bytes, actual_bytes, include_pixels, asyncio.get_event_loop()
    )
    return [ColorEntry(*entry) for entry in entries]

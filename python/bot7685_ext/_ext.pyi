async def wplace_template_compare(
    template_bytes: bytes,
    actual_bytes: bytes,
    include_pixels: bool,
    /,
) -> list[tuple[str, int, int, list[tuple[int, int]]]]: ...

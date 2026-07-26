"""Monkey-patch for nonebot_plugin_htmlrender — remote Playwright support.

When Playwright is deployed as a separate container, `file://` URLs cannot be
resolved by the remote browser (different filesystem).  This patch intercepts
every `file://` template_path / base_url passed through html_to_pic and
get_new_page, replacing them with virtual HTTP URLs that are served from the
*bot-side* filesystem via Playwright's route interception API.

Virtual URL format:
    http://htmlrender-local.bot/{abs_dir_path}/
where:
    abs_dir_path = POSIX absolute path of the template directory

Embedding the absolute directory path as the URL path component lets the
browser resolve relative references (including `../`) correctly:
    ../images/file.png
    → http://htmlrender-local.bot/abs/path/tp/resources/images/file.png
The handler strips the URL path and serves it as an absolute FS path.
All directories share the same virtual host; the absolute path in the URL
path component provides unambiguous file identity without any hashing.

A catch-all proxy route (registered first, lowest priority) intercepts all
external HTTP/HTTPS requests, proxies them via httpx on the bot side, and
adds Access-Control-Allow-Origin: * to avoid CORS errors.  Templates that
reference hundreds of remote images (e.g. skland's operator roster) burst
far past a single connection's capacity, so the proxy bounds concurrency,
retries idempotent requests whose pooled connection was closed by the peer,
and aborts — never falls back — when a request is unrecoverable.

Patch points
------------
* `nonebot_plugin_htmlrender.browser.get_new_page`  — register routes on page
* `nonebot_plugin_htmlrender.data_source.html_to_pic`  — full replacement to
  bypass the original `file:` check.
"""

from __future__ import annotations

import asyncio
import contextlib
import functools
import posixpath
import re
from collections.abc import AsyncGenerator, Callable
from mimetypes import guess_type
from pathlib import Path
from typing import TYPE_CHECKING, Literal
from urllib.parse import quote, unquote, urlparse

import anyio
import httpx
from nonebot.utils import escape_tag, logger_wrapper

if TYPE_CHECKING:
    from nonebot_plugin_htmlrender.browser import get_new_page as _orig_get_new_page
    from playwright.async_api import Page, Request, Response, Route
else:
    _orig_get_new_page = None

type RouteHandler = Callable[[Route, Request], object]
log = logger_wrapper("Patch HTMLRender")

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_PATCH_HTMLRENDER_VERSION = "0.6.7"
_VIRTUAL_HOST = "htmlrender-local.bot"
_VIRTUAL_BASE = f"http://{_VIRTUAL_HOST}"
_VIRTUAL_PATTERN = f"{_VIRTUAL_BASE}/**"
_FILE_URL_IN_HTML_RE = re.compile(r"file://[^\s\"'<>)]+")
_CSS_URL_RE = re.compile(
    r"url\((?P<pre>\s*)(?P<quote>['\"]?)(?P<value>[^\"')]+)(?P=quote)(?P<post>\s*)\)",
    re.IGNORECASE,
)
_WINDOWS_ABS_PATH_RE = re.compile(r"^[a-zA-Z]:[\\/]")
_URL_SCHEME_RE = re.compile(r"^[a-zA-Z][a-zA-Z0-9+.-]*:")

# Headers that must not be relayed as-is when proxying through httpx.
# `accept-encoding` is dropped so httpx negotiates only codecs it can decode
# (the browser advertises br/zstd, which would otherwise come back encoded and
# be handed to the page as identity bytes once `content-encoding` is stripped).
_STRIPPED_REQUEST_HEADERS = frozenset(
    {"host", "origin", "referer", "accept-encoding", "connection", "content-length"}
)
# httpx decodes the body, so the original framing/encoding headers are wrong.
_STRIPPED_RESPONSE_HEADERS = frozenset(
    {"content-encoding", "content-length", "transfer-encoding", "connection"}
)
_PROXY_RETRY_ATTEMPTS = 3
_PROXY_RETRY_METHODS = frozenset({"GET", "HEAD", "OPTIONS"})


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _normalize_abs_path(raw: str) -> str:
    """Lexically normalise an absolute path into POSIX form.

    POSIX: /opt/venv/../lib/x  → /opt/lib/x
    Windows: C:\\Users\\..\\x  → C:/x

    Deliberately lexical: `Path.resolve()` both touches the filesystem and,
    on Windows, anchors drive-less POSIX paths to the current drive — which
    would corrupt every path coming from a Linux-side render.
    """
    normalized = posixpath.normpath(raw.replace("\\", "/"))
    # normpath collapses the leading "//" that POSIX reserves as implementation
    # defined; a bot-side absolute path never means a UNC share here.
    return "/" + normalized.lstrip("/") if raw.startswith(("/", "\\")) else normalized


def _abs_path_to_url_path(path: str) -> str:
    """Convert a normalised absolute path to a URL path component.

    POSIX: /opt/venv/lib/...  → /opt/venv/lib/...
    Windows: C:/Users/...  → /C:/Users/...
    """
    if len(path) >= 2 and path[1] == ":":  # Windows drive letter
        path = "/" + path
    return quote(path, safe="/:")


def _virtual_url_for_path(path: str) -> str:
    """Build the virtual-host URL serving a bot-side absolute path."""
    return f"{_VIRTUAL_BASE}{_abs_path_to_url_path(_normalize_abs_path(path))}"


def _parse_file_url(url: str) -> str | None:
    """Convert a file:// URL to a local path; return None for any other scheme.

    Handles both POSIX paths (file:///app/...) and the Windows form (file://C:/...).
    """
    if not url.startswith("file://"):
        return None
    path_str = unquote(url[7:]).replace("\\", "/")
    # Windows: "file:///C:/foo" → "/C:/foo"; strip the leading "/" when the
    # second char is a drive-letter colon.
    if len(path_str) >= 3 and path_str[0] == "/" and path_str[2] == ":":
        path_str = path_str[1:]
    return _normalize_abs_path(path_str)


async def _file_handler(route: Route, request: Request) -> None:
    """
    Playwright route handler that serves files from the bot-side filesystem
    based on the URL path component.

    The URL path is interpreted as an absolute filesystem path.  This works
    because the virtual URL format embeds the absolute directory path, and
    the browser resolves relative references before the request reaches this
    handler.

    For example, a request to http://htmlrender-local.bot/opt/file.css
    corresponds to the bot-side file at /opt/file.css.
    """
    url_path = unquote(urlparse(request.url).path)
    log("DEBUG", f"Handling virtual file request: <y>{escape_tag(url_path)}</>")

    if not url_path or url_path == "/":
        # page.goto() root navigation — return an empty placeholder so
        # Playwright considers the navigation successful.
        await _fulfill(route, content_type="text/html", body=b"")
        return

    # Convert URL path component back to an absolute filesystem path.
    # POSIX: "/opt/venv/.../file.css" → anyio.Path("/opt/venv/.../file.css")
    # Windows: "/C:/Users/.../file.css" → anyio.Path("C:/Users/.../file.css")
    stripped = url_path.lstrip("/")
    target = anyio.Path(
        stripped  # Windows: "C:/..." is already absolute
        if len(stripped) >= 2 and stripped[1] == ":"
        else "/" + stripped  # POSIX: restore leading "/"
    )

    if not await target.is_file():
        log("DEBUG", f"404: <y>{escape_tag(str(target))}</>")
        await _fulfill(route, status=404)
        return

    mime, _ = guess_type(target.name)
    await _fulfill(
        route,
        body=await target.read_bytes(),
        content_type=mime or "application/octet-stream",
    )


def _file_url_to_virtual(url: str) -> str:
    """Convert a file:// URL to a virtual HTTP URL.

    Returns the URL unchanged if it is not a file:// URL.
    """
    local_path = _parse_file_url(url)
    return url if local_path is None else _virtual_url_for_path(local_path)


def _local_ref_to_virtual(value: str) -> str | None:
    """Virtualise a bare local filesystem path referenced from HTML/CSS.

    Returns None when the value needs no rewriting: fragment references
    (``url(#gradient)``), protocol-relative and absolute URLs, ``data:`` URIs,
    and relative paths — the latter are resolved by the browser against the
    injected ``<base href>`` and hit the virtual-host route anyway.
    """
    ref = value.strip()
    if not ref or ref.startswith(("#", "//")):
        return None

    normalized = ref.replace("\\", "/")
    # Windows drive paths must be tested before the generic scheme check,
    # since "C:/x" also looks like a scheme.
    if _WINDOWS_ABS_PATH_RE.match(normalized):
        return _virtual_url_for_path(normalized)
    if _URL_SCHEME_RE.match(ref):
        return None
    if normalized.startswith("/"):
        return _virtual_url_for_path(normalized)
    return None


def _preprocess_html_file_urls(html: str) -> str:
    """Rewrite local file URLs/paths in HTML/CSS into virtual-host URLs."""
    replaced = 0

    def _replace(match: re.Match[str]) -> str:
        nonlocal replaced
        original = match.group(0)
        converted = _file_url_to_virtual(original)
        if converted != original:
            replaced += 1
        return converted

    processed = _FILE_URL_IN_HTML_RE.sub(_replace, html)

    def _replace_css_url(match: re.Match[str]) -> str:
        nonlocal replaced
        original_value = match.group("value")
        converted = _local_ref_to_virtual(original_value)
        if converted is None or converted == original_value:
            return match.group(0)

        replaced += 1
        return (
            f"url({match.group('pre')}{match.group('quote')}"
            f"{converted}{match.group('quote')}{match.group('post')})"
        )

    processed = _CSS_URL_RE.sub(_replace_css_url, processed)
    if replaced:
        log("DEBUG", f"Replaced {replaced} local URL/path reference(s) in HTML")
    return processed


_PROXY_SEMAPHORE = asyncio.Semaphore(64)
_PROXY_LIMITS = httpx.Limits(
    max_connections=32,
    max_keepalive_connections=16,
    keepalive_expiry=5.0,
)
_PROXY_TIMEOUT = httpx.Timeout(20.0, connect=10.0)
# A pooled HTTP/2 connection torn down by the peer (GOAWAY) surfaces as
# LocalProtocolError on the next request that picks it up; the pool discards
# it, so a retry lands on a fresh connection.
_RETRYABLE_PROXY_EXC = (
    httpx.LocalProtocolError,
    httpx.RemoteProtocolError,
    httpx.ConnectError,
    httpx.ReadError,
    httpx.WriteError,
    httpx.PoolTimeout,
    httpx.ConnectTimeout,
    httpx.ReadTimeout,
)


async def _proxy_fetch(
    client: httpx.AsyncClient,
    request: Request,
) -> httpx.Response:
    headers = {
        k: v
        for k, v in request.headers.items()
        if k.lower() not in _STRIPPED_REQUEST_HEADERS
    }
    retryable = request.method.upper() in _PROXY_RETRY_METHODS
    attempt = 0

    while True:
        attempt += 1
        try:
            async with _PROXY_SEMAPHORE:
                return await client.request(
                    method=request.method,
                    url=request.url,
                    headers=headers,
                    content=request.post_data_buffer,
                )
        except _RETRYABLE_PROXY_EXC as exc:
            if not retryable or attempt >= _PROXY_RETRY_ATTEMPTS:
                raise
            log(
                "DEBUG",
                f"Proxy attempt {attempt} failed for "
                f"<y>{escape_tag(request.url)}</>: {exc!r}, retrying",
            )
            await asyncio.sleep(0.1 * attempt)


async def _proxy_handler(
    client: httpx.AsyncClient,
    route: Route,
    request: Request,
) -> None:
    # Virtual-host URLs are handled by more specific routes registered
    # later (higher priority in Playwright LIFO).  If one reaches here
    # something went wrong — return 404 immediately instead of fallback(),
    # because fallback() on the lowest-priority handler sends the request
    # to the real network, causing a DNS timeout for our virtual hosts.
    if _VIRTUAL_HOST in (urlparse(request.url).hostname or ""):
        log(
            "WARNING",
            "Request for virtual host reached proxy handler (route miss?): "
            f"<y>{escape_tag(request.url)}</>",
        )
        await _fulfill(route, status=404)
        return

    url = request.url
    log("DEBUG", f"Proxying request: <y>{escape_tag(url)}</>")

    try:
        resp = await _proxy_fetch(client, request)
    except Exception as exc:
        # Never fall back to the browser's own network stack: the remote
        # Playwright container has no route to these hosts, so fallback()
        # would stall the render until the navigation timeout.
        log("WARNING", f"Proxy failed for <y>{escape_tag(url)}</>", exc)
        with contextlib.suppress(Exception):
            await route.abort("failed")
        return

    headers = {
        k: v
        for k, v in resp.headers.multi_items()
        if k.lower() not in _STRIPPED_RESPONSE_HEADERS
    }
    headers["access-control-allow-origin"] = "*"
    headers["access-control-allow-credentials"] = "true"
    await _fulfill(route, status=resp.status_code, headers=headers, body=resp.content)


async def _fulfill(
    route: Route,
    *,
    status: int | None = None,
    headers: dict[str, str] | None = None,
    body: bytes | None = None,
    content_type: str | None = None,
) -> None:
    """Fulfill a route, tolerating a page torn down mid-flight."""
    try:
        await route.fulfill(
            status=status,
            headers=headers,
            body=body,
            content_type=content_type,
        )
    except Exception as exc:
        log("DEBUG", f"Failed to fulfill route: {exc!r}")


@contextlib.asynccontextmanager
async def _make_proxy_handler() -> AsyncGenerator[RouteHandler]:
    """Yield a catch-all route handler that proxies external HTTP/HTTPS
    requests through the bot-side httpx client.

    This prevents CORS errors for external resources (CDN fonts, scripts, etc.)
    referenced by templates.  Every response gets Access-Control-Allow-Origin: *
    added so the browser accepts cross-origin subresources.

    Virtual-host requests that slip past the specific route registrations are
    answered with 404 rather than proxied.
    """
    async with httpx.AsyncClient(
        follow_redirects=True,
        http2=True,
        limits=_PROXY_LIMITS,
        timeout=_PROXY_TIMEOUT,
    ) as client:
        yield functools.partial(_proxy_handler, client)


# ---------------------------------------------------------------------------
# Patched get_new_page
# ---------------------------------------------------------------------------


def _patch_page(page: Page) -> None:
    original_goto = page.goto
    original_set_content = page.set_content

    template_path: str | None = None

    # Skip page.goto() to the virtual host URL.
    # Instead inject a <base href> tag so the browser resolves all
    # relative resource references against the virtual template URL.
    # The routes registered above intercept those requests and serve
    # files from the bot-side filesystem.

    async def patched_goto(
        url: str,
        *,
        timeout: float | None = None,  # noqa: ASYNC109
        wait_until: Literal["commit", "domcontentloaded", "load", "networkidle"]
        | None = None,
        referer: str | None = None,
    ) -> Response | None:
        if (local_path := _parse_file_url(url)) is not None:
            nonlocal template_path
            template_path = _virtual_url_for_path(local_path)
            # A directory base must keep its trailing slash, otherwise the
            # browser resolves relative refs against its parent.
            if url.endswith("/") or await anyio.Path(local_path).is_dir():
                template_path += "/"
            log("DEBUG", f"Page navigation to file URL: <y>{escape_tag(url)}</>")
            url = _VIRTUAL_BASE

        return await original_goto(
            url,
            timeout=timeout,
            wait_until=wait_until,
            referer=referer,
        )

    async def set_content(
        html: str,
        *,
        timeout: float | None = None,  # noqa: ASYNC109
        wait_until: Literal["commit", "domcontentloaded", "load", "networkidle"]
        | None = None,
    ) -> None:
        html = _preprocess_html_file_urls(html)

        if template_path is not None:
            base_tag = f'<base href="{template_path}">'
            if m := re.search(r"<head(?:\s[^>]*)?>", html, re.IGNORECASE):
                insert_at = m.end()
                html = html[:insert_at] + base_tag + html[insert_at:]
            else:
                html = base_tag + html

        return await original_set_content(
            html,
            timeout=timeout,
            wait_until=wait_until,
        )

    page.goto = patched_goto
    page.set_content = set_content


@contextlib.asynccontextmanager
async def _patched_get_new_page(
    device_scale_factor: float = 2,
    **kwargs: object,
) -> AsyncGenerator[Page]:
    # Transform base_url when it is a file:// URL.
    # If called from _patched_html_to_pic, base_url was already virtualised
    # there; _file_url_to_virtual() will return it unchanged (not file://).
    if isinstance(base_url := kwargs.get("base_url"), str):
        kwargs["base_url"] = _file_url_to_virtual(base_url)

    async with (
        _orig_get_new_page(device_scale_factor, **kwargs) as page,
        _make_proxy_handler() as proxy_handler,
    ):
        # Proxy catch-all registered first → lowest priority in Playwright LIFO.
        await page.route("**/*", proxy_handler)
        # Virtual host route registered last → highest priority.
        await page.route(_VIRTUAL_PATTERN, _file_handler)
        _patch_page(page)
        yield page


# ---------------------------------------------------------------------------
# Patched html_to_pic
# ---------------------------------------------------------------------------
# The original raises if "file:" is not in template_path, which would reject
# our virtualised "http://" URLs.  We replicate the rendering logic in full
# (it is intentionally minimal) and add the file-route injection.


async def _patched_html_to_pic(
    html: str,
    wait: int = 0,
    template_path: str = f"file://{Path.cwd()}",
    type: Literal["jpeg", "png"] = "png",  # noqa: A002
    quality: int | None = None,
    device_scale_factor: float = 2,
    screenshot_timeout: float | None = 30_000,
    full_page: bool | None = True,
    **kwargs: object,
) -> bytes:
    # Virtualise base_url if present; otherwise fall back to template_path so
    # the browser context always has a virtual base_url set.
    kwargs["base_url"] = _file_url_to_virtual(
        base_url
        if isinstance(base_url := kwargs.get("base_url"), str)
        else template_path,
    )

    async with _patched_get_new_page(device_scale_factor, **kwargs) as page:
        page.on("console", lambda msg: log("DEBUG", f"浏览器控制台: {msg.text}"))
        await page.goto(template_path, wait_until="networkidle")
        await page.set_content(html, wait_until="networkidle")
        await page.wait_for_timeout(wait)
        return await page.screenshot(
            full_page=full_page,
            type=type,
            quality=quality,
            timeout=screenshot_timeout,
        )


# ---------------------------------------------------------------------------
# Patching API
# ---------------------------------------------------------------------------


def patch_htmlrender() -> None:
    from importlib.metadata import version

    htmlrender_version = version("nonebot_plugin_htmlrender")
    log("DEBUG", f"nonebot_plugin_htmlrender version: {htmlrender_version}")
    if htmlrender_version != _PATCH_HTMLRENDER_VERSION:
        log(
            "WARNING",
            f"Expected nonebot_plugin_htmlrender version {_PATCH_HTMLRENDER_VERSION} "
            f"for Playwright patching, but found {htmlrender_version}. "
            "Patching skipped to avoid potential breakage",
        )
        return

    import nonebot_plugin_htmlrender as _htmlrender_mod
    import nonebot_plugin_htmlrender.browser as _browser_mod
    import nonebot_plugin_htmlrender.data_source as _ds_mod

    global _orig_get_new_page
    _orig_get_new_page = _browser_mod.get_new_page

    # browser module — affects every caller that imports get_new_page from there
    _browser_mod.get_new_page = _patched_get_new_page
    # data_source module — the `from ... import get_new_page` binding inside it
    _ds_mod.get_new_page = _patched_get_new_page
    # html_to_pic replacement (template_to_pic calls it by module-level name lookup)
    _ds_mod.html_to_pic = _patched_html_to_pic
    # also patch the re-exports
    _htmlrender_mod.get_new_page = _patched_get_new_page
    _htmlrender_mod.html_to_pic = _patched_html_to_pic

    log("SUCCESS", "Applied htmlrender patches for remote Playwright support")


def register_htmlrender_patch() -> None:
    from .plugin_load import on_plugin_load

    @on_plugin_load("after", plugin_id="nonebot_plugin_htmlrender", skip_on_exc=True)
    def apply_htmlrender_patch(_: object) -> None:
        patch_htmlrender()


__all__ = ["patch_htmlrender", "register_htmlrender_patch"]

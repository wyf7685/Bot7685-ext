from collections.abc import Callable
from importlib.machinery import SourceFileLoader
from types import ModuleType
from typing import Literal, cast, overload
from typing_extensions import deprecated

from nonebot import logger
from nonebot.plugin import Plugin, PluginMetadata, _current_plugin
from nonebot.plugin.manager import PluginLoader

type BeforePluginLoadHook = Callable[[Plugin], object]
type AfterPluginLoadHook = Callable[[Plugin, Exception | None], object]
type AfterPluginLoadSkipExcHook = Callable[[Plugin], object]


_before_plugin_load_hooks: list[tuple[BeforePluginLoadHook, str | None]] = []
_after_plugin_load_hooks: list[
    tuple[AfterPluginLoadHook, str | None, Literal[False]]
    | tuple[AfterPluginLoadSkipExcHook, str | None, Literal[True]]
] = []


@overload
def on_plugin_load[F: BeforePluginLoadHook](
    when: Literal["before"],
    /,
    *,
    plugin_id: str | None = None,
) -> Callable[[F], F]: ...
@overload
def on_plugin_load[F: AfterPluginLoadHook](
    when: Literal["after"],
    /,
    *,
    plugin_id: str | None = None,
) -> Callable[[F], F]: ...
@overload
def on_plugin_load[F: AfterPluginLoadSkipExcHook](
    when: Literal["after"],
    /,
    *,
    plugin_id: str | None = None,
    skip_on_exc: Literal[True],
) -> Callable[[F], F]: ...


def on_plugin_load(
    when: str,
    /,
    *,
    plugin_id: str | None = None,
    skip_on_exc: bool = False,
) -> Callable[..., Callable]:
    def decorator(func: Callable) -> Callable:
        if when == "before":
            _before_plugin_load_hooks.append((func, plugin_id))
        elif when == "after":
            _after_plugin_load_hooks.append((func, plugin_id, skip_on_exc))  # pyright: ignore[reportArgumentType]
        else:
            raise ValueError(f"Invalid hook type: {when!r}")
        return func

    return decorator


@deprecated("Use on_plugin_load('before') instead")
def before_plugin_load[F: BeforePluginLoadHook](func: F) -> F:
    return on_plugin_load("before")(func)


@deprecated("Use on_plugin_load('after') instead")
def after_plugin_load[F: AfterPluginLoadHook](func: F) -> F:
    return on_plugin_load("after")(func)


def _run_before_plugin_load_hooks(plugin: Plugin) -> None:
    for hook, plugin_id in _before_plugin_load_hooks:
        try:
            if plugin_id is None or plugin_id == plugin.id_:
                hook(plugin)
        except Exception:
            logger.exception(
                f"Error in before plugin load hook {hook} for plugin {plugin.id_!r}"
            )


def _run_after_plugin_load_hooks(plugin: Plugin, exc: Exception | None) -> None:
    for hook, plugin_id, skip_on_exc in _after_plugin_load_hooks:
        try:
            if plugin_id is None or plugin_id == plugin.id_:
                if skip_on_exc:
                    if exc is None:
                        cast("AfterPluginLoadSkipExcHook", hook)(plugin)
                else:
                    cast("AfterPluginLoadHook", hook)(plugin, exc)
        except Exception:
            logger.exception(
                f"Error in after plugin load hook {hook} for plugin {plugin.id_!r}"
            )


class HookedLoader(SourceFileLoader):
    def exec_module(self, module: ModuleType) -> None:
        plugin = _current_plugin.get()
        if plugin is None:
            return super().exec_module(module)

        _run_before_plugin_load_hooks(plugin)
        try:
            super().exec_module(module)
        except Exception as exc:
            _run_after_plugin_load_hooks(plugin, exc)
            raise
        else:
            # get plugin metadata
            metadata: PluginMetadata | None = getattr(module, "__plugin_meta__", None)
            plugin.metadata = metadata
            _run_after_plugin_load_hooks(plugin, None)


def mount_plugin_loader_hook() -> None:
    PluginLoader.__bases__ = (HookedLoader,)


def unmount_plugin_loader_hook() -> None:
    PluginLoader.__bases__ = (SourceFileLoader,)


__all__ = [
    "after_plugin_load",
    "before_plugin_load",
    "mount_plugin_loader_hook",
    "on_plugin_load",
    "unmount_plugin_loader_hook",
]

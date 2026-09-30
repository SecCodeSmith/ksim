"""Trace-driven Kubernetes simulator for training and benchmarking autoscalers."""

from importlib.metadata import version

from ksim._native import native_version as _native_version

__version__ = version("ksim")


def native_version() -> str:
    """Return the canonical installed package version."""
    return __version__


__all__ = ["__version__", "native_version", "_native_version"]

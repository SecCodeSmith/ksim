"""Trace-driven Kubernetes simulator for training and benchmarking autoscalers."""

from importlib.metadata import version

from ksim._native import native_version

__version__ = version("ksim")
__all__ = ["__version__", "native_version"]

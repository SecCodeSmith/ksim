"""Python reader/writer for `.ksimtrace`, byte-identical to `ksim_workload::trace::encode`."""

from dataclasses import dataclass
from pathlib import Path

import numpy as np
import numpy.typing as npt

MAGIC = b"KSIMTRC1"
VERSION = 1
HEADER_LEN = 48


@dataclass(frozen=True)
class ServiceSeries:
    """Per-tick demand of one service; the three arrays are float32 of equal length."""

    name: str
    request_rate: npt.NDArray[np.float32]
    cpu_millis: npt.NDArray[np.float32]
    memory_bytes: npt.NDArray[np.float32]


def encode(tick_seconds: int, series: list[ServiceSeries]) -> bytes:
    """Layout documented on the Rust `TraceFile`: 48-byte little-endian header, u16-prefixed
    UTF-8 names, zero padding to a multiple of 4, then request_rate, cpu_millis and
    memory_bytes columns, each f32[service][tick]. Raises ValueError for a non-positive tick,
    no series, unequal lengths or a name longer than 65535 bytes."""
    raise NotImplementedError


def write(path: Path, tick_seconds: int, series: list[ServiceSeries]) -> None:
    """`encode` to `path`, creating parent directories."""
    raise NotImplementedError


def read(path: Path) -> tuple[int, list[ServiceSeries]]:
    """Inverse of `write`; raises ValueError on a bad magic, version or length."""
    raise NotImplementedError

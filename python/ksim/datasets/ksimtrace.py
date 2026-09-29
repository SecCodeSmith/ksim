"""Python reader/writer for `.ksimtrace`, byte-identical to `ksim_workload::trace::encode`."""

import struct
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import numpy.typing as npt

MAGIC = b"KSIMTRC1"
VERSION = 1
HEADER_LEN = 48
_HEADER = struct.Struct("<8sIIIIQQQ")
_MAX_NAME_BYTES = 0xFFFF
_COLUMN_COUNT = 3


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
    if tick_seconds <= 0:
        raise ValueError("tick_seconds must be positive")
    if not series:
        raise ValueError("a trace needs at least one series")
    tick_count = _tick_count(series)
    names = _encode_names(series)
    names_offset = HEADER_LEN
    data_offset = names_offset + _padded(len(names))
    header = _HEADER.pack(
        MAGIC, VERSION, tick_seconds, len(series), tick_count, names_offset, len(names), data_offset
    )
    padding = bytes(data_offset - names_offset - len(names))
    return header + names + padding + _encode_columns(series)


def write(path: Path, tick_seconds: int, series: list[ServiceSeries]) -> None:
    """`encode` to `path`, creating parent directories."""
    payload = encode(tick_seconds, series)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(payload)


def read(path: Path) -> tuple[int, list[ServiceSeries]]:
    """Inverse of `write`; raises ValueError on a bad magic, version or length."""
    raw = path.read_bytes()
    if len(raw) < HEADER_LEN:
        raise ValueError("file shorter than the header")
    magic, version, tick, service_count, tick_count, names_offset, names_len, data_offset = (
        _HEADER.unpack_from(raw)
    )
    if magic != MAGIC:
        raise ValueError("bad magic")
    if version != VERSION:
        raise ValueError(f"unsupported version {version}")
    column_bytes = service_count * tick_count * 4
    if len(raw) != data_offset + _COLUMN_COUNT * column_bytes:
        raise ValueError("file length does not match the header")
    names = _decode_names(raw[names_offset : names_offset + names_len], service_count)
    columns = np.frombuffer(raw, dtype="<f4", offset=data_offset).reshape(
        _COLUMN_COUNT, service_count, tick_count
    )
    series = [
        ServiceSeries(name, *(column[index].astype(np.float32) for column in columns))
        for index, name in enumerate(names)
    ]
    return tick, series


def _padded(length: int) -> int:
    return -(-length // 4) * 4


def _tick_count(series: list[ServiceSeries]) -> int:
    tick_count = len(series[0].request_rate)
    for item in series:
        lengths = {len(item.request_rate), len(item.cpu_millis), len(item.memory_bytes)}
        if lengths != {tick_count}:
            raise ValueError(f"series {item.name!r} has unequal lengths")
    return tick_count


def _encode_names(series: list[ServiceSeries]) -> bytes:
    encoded = b""
    for item in series:
        name = item.name.encode("utf-8")
        if len(name) > _MAX_NAME_BYTES:
            raise ValueError(f"service name {item.name[:16]!r}... is too long")
        encoded += struct.pack("<H", len(name)) + name
    return encoded


def _decode_names(raw: bytes, count: int) -> list[str]:
    names: list[str] = []
    cursor = 0
    for _ in range(count):
        if cursor + 2 > len(raw):
            raise ValueError("truncated name table")
        (length,) = struct.unpack_from("<H", raw, cursor)
        cursor += 2
        if cursor + length > len(raw):
            raise ValueError("truncated name table")
        names.append(raw[cursor : cursor + length].decode("utf-8"))
        cursor += length
    return names


def _encode_columns(series: list[ServiceSeries]) -> bytes:
    columns = (
        np.concatenate([item.request_rate for item in series]),
        np.concatenate([item.cpu_millis for item in series]),
        np.concatenate([item.memory_bytes for item in series]),
    )
    return b"".join(column.astype("<f4").tobytes() for column in columns)

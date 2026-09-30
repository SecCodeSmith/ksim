"""Locate each source's input files inside a directory and load them as a long frame."""

import re
from collections.abc import Callable
from dataclasses import dataclass, fields
from pathlib import Path

import polars as pl

from ksim.datasets import alibaba2018, alibaba2021, azure2019, borg2019
from ksim.datasets.normalize import DAY, Window

_AZURE_INVOCATIONS = "invocations_per_function_md.anon.d*.csv"
_AZURE_DAY = re.compile(r"\.d(\d+)\.csv$")


class MissingInputError(Exception):
    """A required input file was not found."""


@dataclass(frozen=True)
class Units:
    """Unit assumptions forwarded to a loader as keyword arguments; None keeps the loader's
    default. Each source accepts only the fields named in `SUPPORTED_UNITS`."""

    memory_bytes_per_unit: float | None = None
    cpu_millis_per_request: float | None = None
    cpu_millis_per_unit: float | None = None
    container_cpu_millis: float | None = None
    container_memory_bytes: float | None = None


SUPPORTED_UNITS: dict[str, frozenset[str]] = {
    "alibaba2018": frozenset({"memory_bytes_per_unit", "cpu_millis_per_request"}),
    "alibaba2021": frozenset({"container_cpu_millis", "container_memory_bytes"}),
    "azure2019": frozenset(),
    "borg2019": frozenset(
        {"cpu_millis_per_unit", "memory_bytes_per_unit", "cpu_millis_per_request"}
    ),
}


def unit_kwargs(source: str, units: Units) -> dict[str, float]:
    """The fields of `units` that are set, as loader keyword arguments. Raises ValueError naming
    the field when a set field is not supported by `source`."""
    supported = SUPPORTED_UNITS[source]
    set_fields = {
        field.name: value
        for field in fields(units)
        if (value := getattr(units, field.name)) is not None
    }
    for name in set_fields:
        if name not in supported:
            raise ValueError(f"{name} is not supported by source {source!r}")
    return set_fields


def _require(directory: Path, pattern: str) -> list[Path]:
    matches = sorted(directory.glob(pattern))
    if not matches:
        raise MissingInputError(f"no file matching {pattern} in {directory}")
    return matches


def _load_alibaba2018(directory: Path, _window: Window, units: Units) -> pl.DataFrame:
    meta = _require(directory, "container_meta.csv")[0]
    usage = _require(directory, "container_usage.csv")[0]
    return alibaba2018.load(meta, usage, **unit_kwargs("alibaba2018", units))


def _load_alibaba2021(directory: Path, _window: Window, units: Units) -> pl.DataFrame:
    return alibaba2021.load(
        _require(directory, "MSResource*.csv"),
        _require(directory, "MSRTQps*.csv"),
        **unit_kwargs("alibaba2021", units),
    )


def _load_borg2019(directory: Path, _window: Window, units: Units) -> pl.DataFrame:
    return borg2019.load(
        _require(directory, "instance_usage*.csv"), **unit_kwargs("borg2019", units)
    )


def _load_azure2019(directory: Path, window: Window, _units: Units) -> pl.DataFrame:
    frames = [
        _load_azure_day(directory, invocations, day)
        for invocations in _require(directory, _AZURE_INVOCATIONS)
        if window.start_s < (day := _azure_day(invocations)) * DAY
        and (day - 1) * DAY < window.end_s
    ]
    if not frames:
        raise MissingInputError(f"no Azure day overlaps the window in {directory}")
    return pl.concat(frames)


def _azure_day(invocations: Path) -> int:
    match = _AZURE_DAY.search(invocations.name)
    if match is None:
        raise MissingInputError(f"cannot read the day from {invocations.name}")
    return int(match.group(1))


def _load_azure_day(directory: Path, invocations: Path, day: int) -> pl.DataFrame:
    suffix = f".anon.d{day:02d}.csv"
    durations = _require(directory, f"function_durations_percentiles{suffix}")[0]
    memory = next(iter(directory.glob(f"app_memory_percentiles{suffix}")), None)
    return azure2019.load_day(day, invocations, durations, memory)


LOADERS: dict[str, Callable[[Path, Window, Units], pl.DataFrame]] = {
    "alibaba2018": _load_alibaba2018,
    "alibaba2021": _load_alibaba2021,
    "azure2019": _load_azure2019,
    "borg2019": _load_borg2019,
}

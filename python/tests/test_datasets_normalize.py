"""Tests for ksim.datasets.normalize to_series and top_services."""

import numpy as np
import polars as pl
import pytest
from ksim.datasets.ksimtrace import ServiceSeries
from ksim.datasets.normalize import SPLITS, Window, to_series, top_services


class TestToSeries:
    """Test conversion from long-format frame to fixed-tick series."""

    def test_to_series_golden_case(self):
        """Two samples of instance i1 (cpu 100, 300 → mean 200) plus instance i2
        cpu 50 in tick 0 → service sum 250."""
        frame = pl.DataFrame(
            {
                "time_s": [0.0, 0.0, 0.0],
                "service": ["svc", "svc", "svc"],
                "instance": ["i1", "i1", "i2"],
                "request_rate": [1.0, 1.0, 0.5],
                "cpu_millis": [100.0, 300.0, 50.0],
                "memory_bytes": [1024.0, 1024.0, 512.0],
            }
        )
        window = Window(0, 10)
        result = to_series(frame, 10, window)
        assert len(result) == 1
        assert result[0].name == "svc"
        assert len(result[0].cpu_millis) == 1
        assert result[0].cpu_millis[0] == 250.0  # (100 + 300) / 2 + 50

    def test_to_series_forward_fill(self):
        """Ticks without samples carry previous value forward."""
        frame = pl.DataFrame(
            {
                "time_s": [0.0, 20.0],
                "service": ["svc", "svc"],
                "instance": ["i1", "i1"],
                "request_rate": [1.0, 1.0],
                "cpu_millis": [100.0, 200.0],
                "memory_bytes": [1024.0, 2048.0],
            }
        )
        window = Window(0, 30)
        result = to_series(frame, 10, window)
        assert len(result) == 1
        assert len(result[0].cpu_millis) == 3  # ceil(30 / 10)
        assert result[0].cpu_millis[0] == 100.0
        assert result[0].cpu_millis[1] == 100.0  # carried forward
        assert result[0].cpu_millis[2] == 200.0

    def test_to_series_zero_before_first_sample(self):
        """First ticks before any sample are 0."""
        frame = pl.DataFrame(
            {
                "time_s": [10.0],
                "service": ["svc"],
                "instance": ["i1"],
                "request_rate": [1.0],
                "cpu_millis": [100.0],
                "memory_bytes": [1024.0],
            }
        )
        window = Window(0, 20)
        result = to_series(frame, 10, window)
        assert len(result) == 1
        assert len(result[0].cpu_millis) == 2
        assert result[0].cpu_millis[0] == 0.0
        assert result[0].cpu_millis[1] == 100.0

    def test_to_series_window_filtering(self):
        """Only samples within [start_s, end_s) are kept."""
        frame = pl.DataFrame(
            {
                "time_s": [0.0, 10.0, 20.0, 30.0],
                "service": ["svc", "svc", "svc", "svc"],
                "instance": ["i1", "i1", "i1", "i1"],
                "request_rate": [1.0, 1.0, 1.0, 1.0],
                "cpu_millis": [100.0, 200.0, 300.0, 400.0],
                "memory_bytes": [1024.0, 2048.0, 3072.0, 4096.0],
            }
        )
        window = Window(5, 25)
        result = to_series(frame, 10, window)
        assert len(result) == 1
        # Ticks are floor((time_s - start) / tick)
        # time 10 → tick 0, time 20 → tick 1
        assert len(result[0].cpu_millis) == 2
        assert result[0].cpu_millis[0] == 200.0
        assert result[0].cpu_millis[1] == 300.0

    def test_to_series_tick_count(self):
        """Tick count = ceil((end - start) / tick)."""
        frame = pl.DataFrame(
            {
                "time_s": [0.0],
                "service": ["svc"],
                "instance": ["i1"],
                "request_rate": [1.0],
                "cpu_millis": [100.0],
                "memory_bytes": [1024.0],
            }
        )
        window = Window(0, 25)
        result = to_series(frame, 10, window)
        assert len(result[0].cpu_millis) == 3  # ceil(25 / 10)

    def test_to_series_sorted_by_name(self):
        """Services sorted by name."""
        frame = pl.DataFrame(
            {
                "time_s": [0.0, 0.0],
                "service": ["zzz", "aaa"],
                "instance": ["i1", "i1"],
                "request_rate": [1.0, 1.0],
                "cpu_millis": [100.0, 50.0],
                "memory_bytes": [1024.0, 512.0],
            }
        )
        window = Window(0, 10)
        result = to_series(frame, 10, window)
        assert result[0].name == "aaa"
        assert result[1].name == "zzz"

    def test_to_series_float32_dtype(self):
        """Arrays are float32."""
        frame = pl.DataFrame(
            {
                "time_s": [0.0],
                "service": ["svc"],
                "instance": ["i1"],
                "request_rate": [1.0],
                "cpu_millis": [100.0],
                "memory_bytes": [1024.0],
            }
        )
        window = Window(0, 10)
        result = to_series(frame, 10, window)
        assert result[0].request_rate.dtype == np.float32
        assert result[0].cpu_millis.dtype == np.float32
        assert result[0].memory_bytes.dtype == np.float32

    def test_to_series_zero_tick_raises(self):
        """Non-positive tick raises ValueError."""
        frame = pl.DataFrame(
            {
                "time_s": [0.0],
                "service": ["svc"],
                "instance": ["i1"],
                "request_rate": [1.0],
                "cpu_millis": [100.0],
                "memory_bytes": [1024.0],
            }
        )
        window = Window(0, 10)
        with pytest.raises(ValueError):
            to_series(frame, 0, window)

    def test_to_series_empty_window_raises(self):
        """Empty window (start >= end) raises ValueError."""
        frame = pl.DataFrame(
            {
                "time_s": [0.0],
                "service": ["svc"],
                "instance": ["i1"],
                "request_rate": [1.0],
                "cpu_millis": [100.0],
                "memory_bytes": [1024.0],
            }
        )
        window = Window(10, 10)
        with pytest.raises(ValueError):
            to_series(frame, 10, window)


class TestTopServices:
    """Test top_services function."""

    def test_top_services_picks_by_total_cpu(self):
        """Services are picked by total CPU, descending."""
        series = [
            __mock_series("low", cpu=[50.0, 50.0]),  # total 100
            __mock_series("high", cpu=[100.0, 100.0]),  # total 200
            __mock_series("mid", cpu=[70.0, 80.0]),  # total 150
        ]
        result = top_services(series, 2)
        assert len(result) == 2
        assert result[0].name == "high"
        assert result[1].name == "mid"

    def test_top_services_ties_by_name(self):
        """Ties are broken by name."""
        series = [
            __mock_series("z", cpu=[100.0, 100.0]),  # total 200
            __mock_series("a", cpu=[100.0, 100.0]),  # total 200
        ]
        result = top_services(series, 2)
        assert result[0].name == "a"
        assert result[1].name == "z"

    def test_top_services_count_smaller_than_list(self):
        """Count smaller than list length returns count items."""
        series = [
            __mock_series("a", cpu=[100.0]),
            __mock_series("b", cpu=[200.0]),
            __mock_series("c", cpu=[50.0]),
        ]
        result = top_services(series, 2)
        assert len(result) == 2

    def test_top_services_count_greater_than_list(self):
        """Count greater than list length returns all items."""
        series = [__mock_series("a", cpu=[100.0])]
        result = top_services(series, 10)
        assert len(result) == 1


class TestSplitsStructure:
    """Test SPLITS constant structure."""

    def test_splits_has_four_sources(self):
        """SPLITS contains four sources."""
        assert len(SPLITS) == 4
        assert set(SPLITS.keys()) == {
            "alibaba2018",
            "alibaba2021",
            "azure2019",
            "borg2019",
        }

    def test_splits_each_source_has_three_splits(self):
        """Each source has train, validation, and test."""
        for _source, splits in SPLITS.items():
            assert len(splits) == 3
            assert set(splits.keys()) == {"train", "validation", "test"}

    def test_splits_non_overlapping_and_ordered(self):
        """Splits are ordered and non-overlapping."""
        for _source, splits in SPLITS.items():
            train = splits["train"]
            val = splits["validation"]
            test = splits["test"]
            assert train.start_s < train.end_s
            assert val.start_s < val.end_s
            assert test.start_s < test.end_s
            assert train.end_s <= val.start_s
            assert val.end_s <= test.start_s


def __mock_series(name: str, cpu: list[float]):
    """Helper to create a ServiceSeries with given CPU values."""
    return ServiceSeries(
        name=name,
        request_rate=np.array([1.0] * len(cpu), dtype=np.float32),
        cpu_millis=np.array(cpu, dtype=np.float32),
        memory_bytes=np.array([1024.0] * len(cpu), dtype=np.float32),
    )

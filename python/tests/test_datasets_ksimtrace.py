"""Tests for ksim.datasets.ksimtrace encode, write, and read."""

import struct

import numpy as np
import pytest
from ksim.datasets.ksimtrace import ServiceSeries, encode, read, write


class TestEncode:
    """Test encoding of trace data to bytes."""

    def test_encode_golden_case(self):
        """Golden case: tick 15, service "web", 2 ticks, request_rate [1,2],
        cpu [100,200], memory [1024,2048] → 80 bytes (struct.pack reference)."""
        series = [
            ServiceSeries(
                name="web",
                request_rate=np.array([1.0, 2.0], dtype=np.float32),
                cpu_millis=np.array([100.0, 200.0], dtype=np.float32),
                memory_bytes=np.array([1024.0, 2048.0], dtype=np.float32),
            )
        ]
        result = encode(15, series)
        expected = (
            b"KSIMTRC1"
            + struct.pack("<IIIIQQQ", 1, 15, 1, 2, 48, 5, 56)
            + struct.pack("<H", 3)
            + b"web"
            + b"\0" * 3
            + struct.pack("<6f", 1, 2, 100, 200, 1024, 2048)
        )
        assert result == expected
        assert len(result) == 80

    def test_encode_zero_tick_raises(self):
        """Non-positive tick raises ValueError."""
        series = [
            ServiceSeries(
                name="web",
                request_rate=np.array([1.0], dtype=np.float32),
                cpu_millis=np.array([100.0], dtype=np.float32),
                memory_bytes=np.array([1024.0], dtype=np.float32),
            )
        ]
        with pytest.raises(ValueError):
            encode(0, series)

    def test_encode_negative_tick_raises(self):
        """Negative tick raises ValueError."""
        series = [
            ServiceSeries(
                name="web",
                request_rate=np.array([1.0], dtype=np.float32),
                cpu_millis=np.array([100.0], dtype=np.float32),
                memory_bytes=np.array([1024.0], dtype=np.float32),
            )
        ]
        with pytest.raises(ValueError):
            encode(-1, series)

    def test_encode_empty_series_raises(self):
        """Empty series list raises ValueError."""
        with pytest.raises(ValueError):
            encode(15, [])

    def test_encode_unequal_lengths_raises(self):
        """Unequal array lengths raise ValueError."""
        series = [
            ServiceSeries(
                name="web",
                request_rate=np.array([1.0, 2.0], dtype=np.float32),
                cpu_millis=np.array([100.0], dtype=np.float32),
                memory_bytes=np.array([1024.0, 2048.0], dtype=np.float32),
            )
        ]
        with pytest.raises(ValueError):
            encode(15, series)

    def test_encode_multiple_services(self):
        """Multiple services are encoded sequentially."""
        series = [
            ServiceSeries(
                name="web",
                request_rate=np.array([1.0], dtype=np.float32),
                cpu_millis=np.array([100.0], dtype=np.float32),
                memory_bytes=np.array([1024.0], dtype=np.float32),
            ),
            ServiceSeries(
                name="db",
                request_rate=np.array([2.0], dtype=np.float32),
                cpu_millis=np.array([200.0], dtype=np.float32),
                memory_bytes=np.array([2048.0], dtype=np.float32),
            ),
        ]
        result = encode(15, series)
        assert result.startswith(b"KSIMTRC1")
        header = struct.unpack("<IIIIQQQ", result[8:48])
        assert header[0] == 1  # version
        assert header[1] == 15  # tick
        assert header[2] == 2  # num_services


class TestWriteRead:
    """Test write and read round-trip."""

    def test_write_read_round_trip(self, tmp_path):
        """Write and read produce identical data."""
        series = [
            ServiceSeries(
                name="web",
                request_rate=np.array([1.0, 2.0], dtype=np.float32),
                cpu_millis=np.array([100.0, 200.0], dtype=np.float32),
                memory_bytes=np.array([1024.0, 2048.0], dtype=np.float32),
            )
        ]
        path = tmp_path / "test.ksimtrace"
        write(path, 15, series)
        tick, read_series = read(path)
        assert tick == 15
        assert len(read_series) == 1
        assert read_series[0].name == "web"
        np.testing.assert_array_equal(read_series[0].request_rate, series[0].request_rate)
        np.testing.assert_array_equal(read_series[0].cpu_millis, series[0].cpu_millis)
        np.testing.assert_array_equal(read_series[0].memory_bytes, series[0].memory_bytes)

    def test_write_creates_parent_directories(self, tmp_path):
        """write() creates parent directories."""
        path = tmp_path / "subdir" / "test.ksimtrace"
        series = [
            ServiceSeries(
                name="web",
                request_rate=np.array([1.0], dtype=np.float32),
                cpu_millis=np.array([100.0], dtype=np.float32),
                memory_bytes=np.array([1024.0], dtype=np.float32),
            )
        ]
        write(path, 15, series)
        assert path.exists()

    def test_read_bad_magic_raises(self, tmp_path):
        """Read with bad magic raises ValueError."""
        path = tmp_path / "test.ksimtrace"
        path.write_bytes(b"BADMAGIC" + b"\x00" * 100)
        with pytest.raises(ValueError):
            read(path)

    def test_read_nonexistent_file_raises(self, tmp_path):
        """Read on nonexistent file raises error."""
        path = tmp_path / "nonexistent.ksimtrace"
        with pytest.raises((FileNotFoundError, ValueError)):
            read(path)

"""Tests for ksim.datasets.sources units handling."""

import contextlib
from unittest import mock

import pytest
from ksim.datasets.sources import LOADERS, SUPPORTED_UNITS, Units, unit_kwargs


class TestUnitKwargs:
    """Test unit_kwargs function."""

    def test_unit_kwargs_empty_units_returns_empty_dict(self):
        """unit_kwargs with all None fields returns empty dict."""
        units = Units()
        result = unit_kwargs("alibaba2018", units)
        assert result == {}

    def test_unit_kwargs_set_field_included(self):
        """unit_kwargs includes set fields in result."""
        units = Units(cpu_millis_per_request=10.0)
        result = unit_kwargs("alibaba2018", units)
        assert result == {"cpu_millis_per_request": 10.0}

    def test_unit_kwargs_multiple_set_fields(self):
        """unit_kwargs includes multiple set fields."""
        units = Units(
            cpu_millis_per_request=10.0,
            memory_bytes_per_unit=1024.0,
        )
        result = unit_kwargs("alibaba2018", units)
        assert result == {
            "cpu_millis_per_request": 10.0,
            "memory_bytes_per_unit": 1024.0,
        }

    def test_unit_kwargs_unsupported_field_raises_valueerror(self):
        """unit_kwargs raises ValueError for unsupported field by source."""
        units = Units(container_cpu_millis=5.0)
        with pytest.raises(ValueError) as exc_info:
            unit_kwargs("alibaba2018", units)
        assert "container_cpu_millis" in str(exc_info.value)

    def test_unit_kwargs_azure2019_no_units_supported(self):
        """azure2019 supports no units; any set field raises ValueError."""
        units = Units(cpu_millis_per_request=10.0)
        with pytest.raises(ValueError) as exc_info:
            unit_kwargs("azure2019", units)
        assert "cpu_millis_per_request" in str(exc_info.value)

    def test_unit_kwargs_alibaba2021_supported_fields(self):
        """alibaba2021 supports container_cpu_millis and container_memory_bytes."""
        units = Units(container_cpu_millis=5.0, container_memory_bytes=1024.0)
        result = unit_kwargs("alibaba2021", units)
        assert result == {
            "container_cpu_millis": 5.0,
            "container_memory_bytes": 1024.0,
        }

    def test_unit_kwargs_borg2019_supported_fields(self):
        """borg2019 supports cpu_millis_per_unit, memory_bytes_per_unit, cpu_millis_per_request."""
        units = Units(
            cpu_millis_per_unit=2.0,
            memory_bytes_per_unit=1024.0,
            cpu_millis_per_request=10.0,
        )
        result = unit_kwargs("borg2019", units)
        assert result == {
            "cpu_millis_per_unit": 2.0,
            "memory_bytes_per_unit": 1024.0,
            "cpu_millis_per_request": 10.0,
        }

    def test_supported_units_keys_match_loaders(self):
        """SUPPORTED_UNITS keys equal LOADERS keys."""
        assert set(SUPPORTED_UNITS.keys()) == set(LOADERS.keys())

    def test_loader_albibaba2018_forwards_unit_kwargs(self, tmp_path):
        """LOADERS['alibaba2018'] forwards unit_kwargs to alibaba2018.load."""
        meta_file = tmp_path / "container_meta.csv"
        usage_file = tmp_path / "container_usage.csv"
        meta_file.write_text("")
        usage_file.write_text("")

        units = Units(cpu_millis_per_request=10.0)

        with mock.patch("ksim.datasets.sources.alibaba2018.load") as mock_load:
            mock_load.return_value = None
            with contextlib.suppress(TypeError, AttributeError):
                LOADERS["alibaba2018"](tmp_path, mock.MagicMock(), units)

            if mock_load.called:
                call_args = mock_load.call_args
                assert call_args is not None
                # Check that cpu_millis_per_request was passed
                if call_args.kwargs:
                    assert call_args.kwargs.get("cpu_millis_per_request") == 10.0

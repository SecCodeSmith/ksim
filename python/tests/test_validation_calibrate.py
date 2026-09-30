"""Tests for ksim.validation.calibrate: cpu_millis_per_request and rewrite_cpu_per_request."""

import pytest
from ksim.validation.calibrate import cpu_millis_per_request, rewrite_cpu_per_request


class TestCpuMillisPerRequest:
    """Test cpu_millis_per_request function."""

    def test_cpu_millis_per_request_basic(self):
        """Calculate CPU per request."""
        result = cpu_millis_per_request(cpu_millicores=500.0, rps=100.0)
        assert result == pytest.approx(5.0)

    def test_cpu_millis_per_request_division(self):
        """Divide cpu_millicores by rps."""
        result = cpu_millis_per_request(cpu_millicores=1000.0, rps=50.0)
        assert result == pytest.approx(20.0)

    def test_cpu_millis_per_request_fractional(self):
        """Handle fractional results."""
        result = cpu_millis_per_request(cpu_millicores=100.0, rps=3.0)
        assert result == pytest.approx(100.0 / 3.0)

    def test_cpu_millis_per_request_zero_rps_error(self):
        """Raise ValueError for zero rps."""
        with pytest.raises(ValueError):
            cpu_millis_per_request(cpu_millicores=500.0, rps=0.0)

    def test_cpu_millis_per_request_negative_rps_error(self):
        """Raise ValueError for negative rps."""
        with pytest.raises(ValueError):
            cpu_millis_per_request(cpu_millicores=500.0, rps=-10.0)

    def test_cpu_millis_per_request_zero_cpu_error(self):
        """Raise ValueError for zero cpu_millicores."""
        with pytest.raises(ValueError):
            cpu_millis_per_request(cpu_millicores=0.0, rps=100.0)

    def test_cpu_millis_per_request_negative_cpu_error(self):
        """Raise ValueError for negative cpu_millicores."""
        with pytest.raises(ValueError):
            cpu_millis_per_request(cpu_millicores=-500.0, rps=100.0)


class TestRewriteCpuPerRequest:
    """Test rewrite_cpu_per_request function."""

    def test_rewrite_cpu_per_request_top_level(self):
        """Rewrite top-level cpu_millis_per_request assignment."""
        toml = 'cpu_millis_per_request = 50.0\nother_key = "value"'
        result = rewrite_cpu_per_request(toml, 75.0)
        assert "cpu_millis_per_request = 75.0" in result
        assert 'other_key = "value"' in result

    def test_rewrite_cpu_per_request_inline_table(self):
        """Rewrite cpu_millis_per_request inside inline tables."""
        toml = (
            "[service]\n"
            'config = { kind = "queueing", cpu_millis_per_request = 50.0, timeout = 10 }\n'
        )
        result = rewrite_cpu_per_request(toml, 75.0)
        assert "cpu_millis_per_request = 75.0" in result
        assert 'kind = "queueing"' in result
        assert "timeout = 10" in result

    def test_rewrite_cpu_per_request_multiple_occurrences(self):
        """Replace all cpu_millis_per_request assignments."""
        toml = (
            "cpu_millis_per_request = 50.0\n[service]\nconfig = { cpu_millis_per_request = 50.0 }\n"
        )
        result = rewrite_cpu_per_request(toml, 75.0)
        assert result.count("cpu_millis_per_request = 75.0") == 2

    def test_rewrite_cpu_per_request_leaves_other_text_unchanged(self):
        """Leave all other text byte-identical."""
        toml = (
            "# This is a comment\n"
            "[section]\n"
            "key = 123\n"
            "cpu_millis_per_request = 50.0\n"
            'another = "value"\n'
        )
        result = rewrite_cpu_per_request(toml, 75.0)
        assert "# This is a comment" in result
        assert "[section]" in result
        assert "key = 123" in result
        assert 'another = "value"' in result

    def test_rewrite_cpu_per_request_uses_repr(self):
        """Write value via repr."""
        result = rewrite_cpu_per_request("cpu_millis_per_request = 50.0", 100.5)
        assert "100.5" in result

    def test_rewrite_cpu_per_request_no_assignment_error(self):
        """Raise ValueError when no assignment found."""
        toml = 'other_key = 50.0\nmore_keys = "value"'
        with pytest.raises(ValueError):
            rewrite_cpu_per_request(toml, 75.0)

    def test_rewrite_cpu_per_request_negative_value_error(self):
        """Raise ValueError for negative value."""
        toml = "cpu_millis_per_request = 50.0"
        with pytest.raises(ValueError):
            rewrite_cpu_per_request(toml, -75.0)

    def test_rewrite_cpu_per_request_zero_value_error(self):
        """Raise ValueError for zero value."""
        toml = "cpu_millis_per_request = 50.0"
        with pytest.raises(ValueError):
            rewrite_cpu_per_request(toml, 0.0)

    def test_rewrite_cpu_per_request_preserves_formatting(self):
        """Preserve TOML formatting and structure."""
        toml = (
            "[api]\n"
            "cpu_millis_per_request = 50.0\n"
            'description = "API service"\n'
            "\n"
            "[web]\n"
            "cpu_millis_per_request = 60.0\n"
        )
        result = rewrite_cpu_per_request(toml, 75.0)
        assert "[api]" in result
        assert "[web]" in result
        assert 'description = "API service"' in result

    def test_rewrite_cpu_per_request_whitespace_handling(self):
        """Handle various whitespace around assignment."""
        toml = "cpu_millis_per_request   =   50.0"
        result = rewrite_cpu_per_request(toml, 75.0)
        assert "cpu_millis_per_request" in result
        assert "75.0" in result

    def test_rewrite_cpu_per_request_with_comments(self):
        """Preserve comments."""
        toml = (
            "# CPU cost per request\n"
            "cpu_millis_per_request = 50.0\n"
            "# Another setting\n"
            'other = "value"\n'
        )
        result = rewrite_cpu_per_request(toml, 75.0)
        assert "# CPU cost per request" in result
        assert "# Another setting" in result
        assert "cpu_millis_per_request = 75.0" in result

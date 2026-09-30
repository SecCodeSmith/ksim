"""Tests for ksim.validation.k6: stages and render_script."""

import pytest
from ksim.validation.k6 import Stage, render_script, stages


class TestStages:
    """Test stages function."""

    def test_stages_basic(self):
        """Convert demand series to stages with default scale."""
        demand = [100.0, 110.0, 120.0]
        result = stages(demand, tick_s=5)
        assert len(result) == 3
        assert result[0] == Stage(duration_s=5, target_rps=100)
        assert result[1] == Stage(duration_s=5, target_rps=110)
        assert result[2] == Stage(duration_s=5, target_rps=120)

    def test_stages_with_scale(self):
        """Apply rate scale factor to demand."""
        demand = [100.0, 110.0, 120.0]
        result = stages(demand, tick_s=5, rate_scale=0.5)
        assert len(result) == 3
        assert result[0].target_rps == 50
        assert result[1].target_rps == 55
        assert result[2].target_rps == 60

    def test_stages_rounding(self):
        """Round target_rps to nearest integer."""
        demand = [100.3, 110.6, 120.1]
        result = stages(demand, tick_s=5)
        assert result[0].target_rps == 100
        assert result[1].target_rps == 111
        assert result[2].target_rps == 120

    def test_stages_never_negative(self):
        """Never produce negative target_rps."""
        demand = [-5.0, 0.0, 5.0]
        result = stages(demand, tick_s=5)
        assert result[0].target_rps == 0
        assert result[1].target_rps == 0
        assert result[2].target_rps == 5

    def test_stages_negative_scale_error(self):
        """Raise ValueError for negative rate_scale."""
        demand = [100.0, 110.0]
        with pytest.raises(ValueError):
            stages(demand, tick_s=5, rate_scale=-0.5)

    def test_stages_non_positive_tick_error(self):
        """Raise ValueError for non-positive tick_s."""
        demand = [100.0, 110.0]
        with pytest.raises(ValueError):
            stages(demand, tick_s=0)
        with pytest.raises(ValueError):
            stages(demand, tick_s=-5)

    def test_stages_empty_demand_error(self):
        """Raise ValueError for empty demand sequence."""
        with pytest.raises(ValueError):
            stages([], tick_s=5)

    def test_stages_tick_s_propagated(self):
        """Use tick_s value in all stages."""
        demand = [100.0, 110.0]
        result = stages(demand, tick_s=10)
        assert all(s.duration_s == 10 for s in result)


class TestRenderScript:
    """Test render_script function."""

    def test_render_script_basic(self):
        """Render k6 script with ramping-arrival-rate executor."""
        test_stages = [Stage(duration_s=5, target_rps=100), Stage(duration_s=5, target_rps=120)]
        script = render_script(test_stages, url="https://api.example.com")
        assert "import http from 'k6/http'" in script
        assert "export const options" in script
        assert "ramping-arrival-rate" in script
        assert "https://api.example.com" in script
        assert "http.get" in script

    def test_render_script_contains_executor(self):
        """Script contains executor: 'ramping-arrival-rate'."""
        test_stages = [Stage(duration_s=5, target_rps=100)]
        script = render_script(test_stages, url="https://example.com")
        assert "executor: 'ramping-arrival-rate'" in script

    def test_render_script_contains_time_unit(self):
        """Script contains timeUnit: '1s'."""
        test_stages = [Stage(duration_s=5, target_rps=100)]
        script = render_script(test_stages, url="https://example.com")
        assert "timeUnit: '1s'" in script

    def test_render_script_start_rate(self):
        """startRate equals first stage's target."""
        test_stages = [
            Stage(duration_s=5, target_rps=100),
            Stage(duration_s=5, target_rps=120),
        ]
        script = render_script(test_stages, url="https://example.com")
        assert "startRate: 100" in script

    def test_render_script_preallocated_vus(self):
        """preAllocatedVUs = max(1, max_vus // 4)."""
        test_stages = [Stage(duration_s=5, target_rps=100)]
        script = render_script(test_stages, url="https://example.com", max_vus=200)
        assert "preAllocatedVUs: 50" in script

    def test_render_script_preallocated_vus_minimum(self):
        """preAllocatedVUs at least 1."""
        test_stages = [Stage(duration_s=5, target_rps=100)]
        script = render_script(test_stages, url="https://example.com", max_vus=2)
        assert "preAllocatedVUs: 1" in script

    def test_render_script_max_vus(self):
        """maxVUs equals max_vus parameter."""
        test_stages = [Stage(duration_s=5, target_rps=100)]
        script = render_script(test_stages, url="https://example.com", max_vus=300)
        assert "maxVUs: 300" in script

    def test_render_script_stages_format(self):
        """Stages rendered as { duration: '5s', target: 100 }."""
        test_stages = [
            Stage(duration_s=5, target_rps=100),
            Stage(duration_s=10, target_rps=120),
        ]
        script = render_script(test_stages, url="https://example.com")
        assert "duration: '5s'" in script
        assert "duration: '10s'" in script
        assert "target: 100" in script
        assert "target: 120" in script

    def test_render_script_export_default_function(self):
        """Contains export default function."""
        test_stages = [Stage(duration_s=5, target_rps=100)]
        script = render_script(test_stages, url="https://example.com")
        assert "export default function" in script
        assert "http.get" in script

    def test_render_script_no_stages_error(self):
        """Raise ValueError for empty stages."""
        with pytest.raises(ValueError):
            render_script([], url="https://example.com")

    def test_render_script_invalid_max_vus_error(self):
        """Raise ValueError for max_vus < 1."""
        test_stages = [Stage(duration_s=5, target_rps=100)]
        with pytest.raises(ValueError):
            render_script(test_stages, url="https://example.com", max_vus=0)
        with pytest.raises(ValueError):
            render_script(test_stages, url="https://example.com", max_vus=-1)

    def test_render_script_scenario_name_replay(self):
        """Scenario named 'replay'."""
        test_stages = [Stage(duration_s=5, target_rps=100)]
        script = render_script(test_stages, url="https://example.com")
        assert "replay:" in script

    def test_render_script_url_in_http_get(self):
        """URL appears in http.get call."""
        url = "https://api.example.com/endpoint"
        test_stages = [Stage(duration_s=5, target_rps=100)]
        script = render_script(test_stages, url=url)
        assert url in script

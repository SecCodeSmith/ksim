from datetime import UTC, datetime, timedelta

import pytest
from ksim_ci.nightly_detect import branches_to_build


class TestBranchesToBuild:
    @pytest.fixture
    def utc(self):
        return UTC

    def test_main_committed_within_window_included(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        last_commit = {
            "main": datetime(2026, 9, 29, 11, 0, 0, tzinfo=utc),
        }
        result = branches_to_build(last_commit, now, timedelta(hours=24))
        assert "main" in result

    def test_main_committed_outside_window_excluded(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        last_commit = {
            "main": datetime(2026, 9, 27, 11, 0, 0, tzinfo=utc),
        }
        result = branches_to_build(last_commit, now, timedelta(hours=24))
        assert "main" not in result

    def test_release_branch_within_window_included(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        last_commit = {
            "release/1.2": datetime(2026, 9, 29, 10, 0, 0, tzinfo=utc),
        }
        result = branches_to_build(last_commit, now, timedelta(hours=24))
        assert "release/1.2" in result

    def test_feature_branch_excluded_regardless_of_window(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        last_commit = {
            "feature/x": datetime(2026, 9, 29, 11, 0, 0, tzinfo=utc),
        }
        result = branches_to_build(last_commit, now, timedelta(hours=24))
        assert "feature/x" not in result

    def test_releases_misspelling_excluded(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        last_commit = {
            "releases/1.2": datetime(2026, 9, 29, 11, 0, 0, tzinfo=utc),
        }
        result = branches_to_build(last_commit, now, timedelta(hours=24))
        assert "releases/1.2" not in result

    def test_empty_dict_returns_empty_list(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        result = branches_to_build({}, now, timedelta(hours=24))
        assert result == []

    def test_commit_exactly_at_window_boundary_excluded(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        window = timedelta(hours=24)
        last_commit = {
            "main": now - window,
        }
        result = branches_to_build(last_commit, now, window)
        assert "main" not in result

    def test_result_is_sorted(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        last_commit = {
            "release/2.0": datetime(2026, 9, 29, 11, 0, 0, tzinfo=utc),
            "main": datetime(2026, 9, 29, 11, 0, 0, tzinfo=utc),
            "release/1.0": datetime(2026, 9, 29, 11, 0, 0, tzinfo=utc),
        }
        result = branches_to_build(last_commit, now, timedelta(hours=24))
        assert result == ["main", "release/1.0", "release/2.0"]

    def test_multiple_branches_mixed_eligibility(self, utc):
        now = datetime(2026, 9, 29, 12, 0, 0, tzinfo=utc)
        last_commit = {
            "main": datetime(2026, 9, 29, 11, 0, 0, tzinfo=utc),
            "release/1.2": datetime(2026, 9, 29, 10, 0, 0, tzinfo=utc),
            "feature/x": datetime(2026, 9, 29, 11, 0, 0, tzinfo=utc),
            "release/2.0": datetime(2026, 9, 27, 11, 0, 0, tzinfo=utc),
        }
        result = branches_to_build(last_commit, now, timedelta(hours=24))
        assert set(result) == {"main", "release/1.2"}
        assert result == sorted(result)

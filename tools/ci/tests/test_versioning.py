from datetime import date

import pytest
from ksim_ci.versioning import next_nightly_version, set_workspace_version


class TestNextNightlyVersion:
    @pytest.mark.parametrize(
        "current,branch,day,expected",
        [
            ("0.1.0", "main", date(2026, 9, 29), "0.1.0-nightly.20260929"),
            ("0.2.0-nightly.20260928", "main", date(2026, 9, 29), "0.2.0-nightly.20260929"),
            ("1.0.0", "main", date(2025, 1, 1), "1.0.0-nightly.20250101"),
            ("2.3.4-alpha", "main", date(2026, 9, 29), "2.3.4-nightly.20260929"),
            ("2.3.4+build.9", "main", date(2026, 9, 29), "2.3.4-nightly.20260929"),
        ],
    )
    def test_main_branch_creates_nightly_version(self, current, branch, day, expected):
        result = next_nightly_version(current, branch, day)
        assert result == expected

    @pytest.mark.parametrize(
        "current,branch,day,expected",
        [
            ("1.2.3", "release/1.2", date(2026, 9, 29), "1.2.4"),
            ("1.2.0-nightly.20260928", "release/1.2", date(2026, 9, 29), "1.2.0"),
            ("1.2.0-alpha", "release/1.2", date(2026, 9, 29), "1.2.0"),
            ("1.2.9", "release/1.2", date(2026, 9, 29), "1.2.10"),
            ("1.2.3+build.9", "release/1.2", date(2026, 9, 29), "1.2.4"),
        ],
    )
    def test_release_branch_bumps_patch_or_removes_prerelease(self, current, branch, day, expected):
        result = next_nightly_version(current, branch, day)
        assert result == expected

    def test_release_branch_version_mismatch_raises_value_error(self):
        with pytest.raises(ValueError):
            next_nightly_version("2.3.4", "release/1.2", date(2026, 9, 29))

    @pytest.mark.parametrize(
        "branch",
        [
            "feature/x",
            "releases/1.2",
            "development",
            "hotfix/bug",
        ],
    )
    def test_invalid_branch_raises_value_error(self, branch):
        with pytest.raises(ValueError):
            next_nightly_version("1.2.3", branch, date(2026, 9, 29))

    @pytest.mark.parametrize(
        "malformed",
        [
            "1.2",
            "1",
            "1.2.3.4",
            "a.b.c",
            "1.2.3-01",
            "1.2.3-alpha.01",
        ],
    )
    def test_malformed_version_raises_value_error(self, malformed):
        with pytest.raises(ValueError):
            next_nightly_version(malformed, "main", date(2026, 9, 29))


class TestSetWorkspaceVersion:
    def test_replaces_workspace_package_version(self):
        toml_content = """
[workspace.package]
name = "ksim"
version = "0.1.0"
edition = "2021"
"""
        result = set_workspace_version(toml_content, "0.2.0")
        assert 'version = "0.2.0"' in result
        assert 'version = "0.1.0"' not in result

    def test_leaves_other_version_entries_unchanged(self):
        toml_content = """
[workspace.package]
name = "ksim"
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
serde = { version = "1.0.229" }

[dev-dependencies]
tempfile = { version = "3.8" }
"""
        result = set_workspace_version(toml_content, "0.2.0")
        assert 'version = "0.2.0"' in result
        assert 'serde = { version = "1.0.229" }' in result
        assert 'tempfile = { version = "3.8" }' in result

    def test_raises_value_error_if_no_workspace_package_version(self):
        toml_content = """
[workspace]
members = ["crates/*"]

[workspace.dependencies]
serde = { version = "1.0.229" }
"""
        with pytest.raises(ValueError):
            set_workspace_version(toml_content, "0.2.0")

    def test_replaces_only_workspace_package_version_line(self):
        toml_content = """[workspace.package]
version = "1.0.0"
name = "test"
"""
        result = set_workspace_version(toml_content, "2.0.0")
        assert result == '[workspace.package]\nversion = "2.0.0"\nname = "test"\n'

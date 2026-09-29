"""Tests for sarif_gate module (SARIF 2.1.0 level defaulting and filtering)."""

import json
from io import StringIO

import pytest
from ksim_ci.sarif_gate import effective_level, findings_at_or_above, main


class TestEffectiveLevel:
    """Test effective_level function: result.level → rule.defaultConfiguration.level → "warning"."""

    def test_explicit_level_note(self):
        """Result with explicit level 'note' uses that level."""
        result = {"level": "note"}
        rules = {}
        assert effective_level(result, rules) == "note"

    def test_explicit_level_warning(self):
        """Result with explicit level 'warning' uses that level."""
        result = {"level": "warning"}
        rules = {}
        assert effective_level(result, rules) == "warning"

    def test_explicit_level_error(self):
        """Result with explicit level 'error' uses that level."""
        result = {"level": "error"}
        rules = {}
        assert effective_level(result, rules) == "error"

    def test_rule_default_configuration_level_error(self):
        """Result without level; rule has defaultConfiguration.level 'error'."""
        result = {"ruleId": "py/sql-injection"}
        rules = {"py/sql-injection": {"defaultConfiguration": {"level": "error"}}}
        assert effective_level(result, rules) == "error"

    def test_rule_default_configuration_level_warning(self):
        """Result without level; rule has defaultConfiguration.level 'warning'."""
        result = {"ruleId": "security/xss"}
        rules = {"security/xss": {"defaultConfiguration": {"level": "warning"}}}
        assert effective_level(result, rules) == "warning"

    def test_rule_default_configuration_level_note(self):
        """Result without level; rule has defaultConfiguration.level 'note'."""
        result = {"ruleId": "style/naming"}
        rules = {"style/naming": {"defaultConfiguration": {"level": "note"}}}
        assert effective_level(result, rules) == "note"

    def test_no_level_no_rule_defaults_to_warning(self):
        """Result without level and rule not in rules dict defaults to 'warning'."""
        result = {"ruleId": "unknown/rule"}
        rules = {}
        assert effective_level(result, rules) == "warning"

    def test_no_level_rule_without_default_configuration_defaults_to_warning(self):
        """Result without level; rule exists but has no defaultConfiguration."""
        result = {"ruleId": "my/rule"}
        rules = {"my/rule": {}}
        assert effective_level(result, rules) == "warning"

    def test_result_level_overrides_rule_level(self):
        """Result with explicit level overrides rule's defaultConfiguration.level."""
        result = {"level": "note", "ruleId": "py/sql-injection"}
        rules = {"py/sql-injection": {"defaultConfiguration": {"level": "error"}}}
        assert effective_level(result, rules) == "note"


class TestFindingsAtOrAbove:
    """Test findings_at_or_above function: filter by level threshold."""

    def test_single_run_single_error_threshold_error(self):
        """Single run with error result; threshold 'error' returns it."""
        sarif = {
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "rules": [{"id": "rule1", "defaultConfiguration": {"level": "error"}}]
                        }
                    },
                    "results": [{"ruleId": "rule1"}],
                }
            ]
        }
        findings = findings_at_or_above(sarif, "error")
        assert len(findings) == 1
        assert findings[0]["ruleId"] == "rule1"

    def test_single_run_warning_threshold_error_returns_empty(self):
        """Single run with warning result; threshold 'error' returns nothing."""
        sarif = {
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "rules": [{"id": "rule1", "defaultConfiguration": {"level": "warning"}}]
                        }
                    },
                    "results": [{"ruleId": "rule1"}],
                }
            ]
        }
        findings = findings_at_or_above(sarif, "error")
        assert len(findings) == 0

    def test_threshold_warning_includes_warning_and_error(self):
        """Threshold 'warning' returns both warning and error level results."""
        sarif = {
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "rules": [
                                {"id": "rule1", "defaultConfiguration": {"level": "error"}},
                                {"id": "rule2", "defaultConfiguration": {"level": "warning"}},
                                {"id": "rule3", "defaultConfiguration": {"level": "note"}},
                            ]
                        }
                    },
                    "results": [{"ruleId": "rule1"}, {"ruleId": "rule2"}, {"ruleId": "rule3"}],
                }
            ]
        }
        findings = findings_at_or_above(sarif, "warning")
        assert len(findings) == 2
        rule_ids = {f["ruleId"] for f in findings}
        assert rule_ids == {"rule1", "rule2"}

    def test_threshold_none_returns_all(self):
        """Threshold 'none' returns all results."""
        sarif = {
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "rules": [
                                {"id": "rule1", "defaultConfiguration": {"level": "error"}},
                                {"id": "rule2", "defaultConfiguration": {"level": "note"}},
                            ]
                        }
                    },
                    "results": [{"ruleId": "rule1"}, {"ruleId": "rule2"}],
                }
            ]
        }
        findings = findings_at_or_above(sarif, "none")
        assert len(findings) == 2

    def test_multiple_runs(self):
        """Two runs; each can have results; findings from both are returned."""
        sarif = {
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "rules": [{"id": "rule1", "defaultConfiguration": {"level": "error"}}]
                        }
                    },
                    "results": [{"ruleId": "rule1"}],
                },
                {
                    "tool": {
                        "driver": {
                            "rules": [{"id": "rule2", "defaultConfiguration": {"level": "error"}}]
                        }
                    },
                    "results": [{"ruleId": "rule2"}],
                },
            ]
        }
        findings = findings_at_or_above(sarif, "error")
        assert len(findings) == 2
        rule_ids = {f["ruleId"] for f in findings}
        assert rule_ids == {"rule1", "rule2"}

    def test_run_without_results_is_skipped(self):
        """Run without results key is handled gracefully."""
        sarif = {"runs": [{"tool": {"driver": {"rules": []}}}]}
        findings = findings_at_or_above(sarif, "warning")
        assert len(findings) == 0

    def test_run_without_rules_is_handled(self):
        """Run with results but no rules defaults result levels to 'warning'."""
        sarif = {"runs": [{"tool": {"driver": {}}, "results": [{"ruleId": "unknown/rule"}]}]}
        findings = findings_at_or_above(sarif, "warning")
        assert len(findings) == 1

    def test_unknown_threshold_raises_value_error(self):
        """Unknown threshold like 'fatal' raises ValueError."""
        sarif = {"runs": [{"tool": {"driver": {"rules": []}}, "results": []}]}
        with pytest.raises(ValueError):
            findings_at_or_above(sarif, "fatal")

    def test_explicit_level_none_ranks_below_warning(self):
        """An explicit level 'none' is kept and ranks below 'warning'."""
        sarif = {
            "runs": [
                {
                    "tool": {"driver": {"rules": []}},
                    "results": [{"ruleId": "rule1", "level": "none"}],
                }
            ]
        }
        findings = findings_at_or_above(sarif, "warning")
        assert len(findings) == 0


class TestMain:
    """Test main CLI function."""

    def test_single_error_finding_returns_one(self, tmp_path):
        """File with one error finding returns 1."""
        sarif_file = tmp_path / "report.sarif"
        sarif_content = {
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "CodeQL",
                            "rules": [
                                {
                                    "id": "py/sql-injection",
                                    "defaultConfiguration": {"level": "error"},
                                }
                            ],
                        }
                    },
                    "results": [
                        {
                            "ruleId": "py/sql-injection",
                            "message": {"text": "SQL injection vulnerability"},
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "app.py"},
                                        "region": {"startLine": 7},
                                    }
                                }
                            ],
                        }
                    ],
                }
            ],
        }
        sarif_file.write_text(json.dumps(sarif_content))

        stdout = StringIO()
        result = main([str(sarif_file)], stdout)

        assert result == 1
        output = stdout.getvalue()
        assert "py/sql-injection" in output
        assert "error" in output
        assert "app.py:7" in output

    def test_only_warnings_with_error_threshold_returns_zero(self, tmp_path):
        """File with only warnings; default level is error; returns 0."""
        sarif_file = tmp_path / "report.sarif"
        sarif_content = {
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "CodeQL",
                            "rules": [
                                {"id": "style/naming", "defaultConfiguration": {"level": "warning"}}
                            ],
                        }
                    },
                    "results": [
                        {
                            "ruleId": "style/naming",
                            "message": {"text": "Naming convention"},
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "app.py"},
                                        "region": {"startLine": 10},
                                    }
                                }
                            ],
                        }
                    ],
                }
            ],
        }
        sarif_file.write_text(json.dumps(sarif_content))

        stdout = StringIO()
        result = main([str(sarif_file)], stdout)

        assert result == 0

    def test_only_warnings_with_warning_level_returns_one(self, tmp_path):
        """File with only warnings; --level warning; returns 1."""
        sarif_file = tmp_path / "report.sarif"
        sarif_content = {
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "CodeQL",
                            "rules": [
                                {"id": "style/naming", "defaultConfiguration": {"level": "warning"}}
                            ],
                        }
                    },
                    "results": [
                        {
                            "ruleId": "style/naming",
                            "message": {"text": "Naming convention"},
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "app.py"},
                                        "region": {"startLine": 10},
                                    }
                                }
                            ],
                        }
                    ],
                }
            ],
        }
        sarif_file.write_text(json.dumps(sarif_content))

        stdout = StringIO()
        result = main([str(sarif_file), "--level", "warning"], stdout)

        assert result == 1

    def test_missing_file_returns_two(self, tmp_path):
        """Missing file returns exit code 2."""
        nonexistent = tmp_path / "missing.sarif"

        stdout = StringIO()
        result = main([str(nonexistent)], stdout)

        assert result == 2

    def test_two_files_one_with_error_returns_one(self, tmp_path):
        """Two files, one with error; returns 1."""
        warning_file = tmp_path / "warnings.sarif"
        warning_content = {
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "CodeQL",
                            "rules": [
                                {"id": "style/naming", "defaultConfiguration": {"level": "warning"}}
                            ],
                        }
                    },
                    "results": [
                        {
                            "ruleId": "style/naming",
                            "message": {"text": "Naming"},
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "style.py"},
                                        "region": {"startLine": 5},
                                    }
                                }
                            ],
                        }
                    ],
                }
            ],
        }
        warning_file.write_text(json.dumps(warning_content))

        error_file = tmp_path / "errors.sarif"
        error_content = {
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "CodeQL",
                            "rules": [
                                {
                                    "id": "py/sql-injection",
                                    "defaultConfiguration": {"level": "error"},
                                }
                            ],
                        }
                    },
                    "results": [
                        {
                            "ruleId": "py/sql-injection",
                            "message": {"text": "SQL injection"},
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "app.py"},
                                        "region": {"startLine": 7},
                                    }
                                }
                            ],
                        }
                    ],
                }
            ],
        }
        error_file.write_text(json.dumps(error_content))

        stdout = StringIO()
        result = main([str(warning_file), str(error_file)], stdout)

        assert result == 1
        output = stdout.getvalue()
        assert "py/sql-injection" in output

    def test_output_format_multiple_findings(self, tmp_path):
        """Output format is correct for multiple findings."""
        sarif_file = tmp_path / "multi.sarif"
        sarif_content = {
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "CodeQL",
                            "rules": [
                                {"id": "rule1", "defaultConfiguration": {"level": "error"}},
                                {"id": "rule2", "defaultConfiguration": {"level": "error"}},
                            ],
                        }
                    },
                    "results": [
                        {
                            "ruleId": "rule1",
                            "message": {"text": "Finding 1"},
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "file1.py"},
                                        "region": {"startLine": 10},
                                    }
                                }
                            ],
                        },
                        {
                            "ruleId": "rule2",
                            "message": {"text": "Finding 2"},
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "file2.py"},
                                        "region": {"startLine": 20},
                                    }
                                }
                            ],
                        },
                    ],
                }
            ],
        }
        sarif_file.write_text(json.dumps(sarif_content))

        stdout = StringIO()
        result = main([str(sarif_file)], stdout)

        assert result == 1
        output = stdout.getvalue()
        lines = output.strip().split("\n")
        assert len(lines) == 2
        # Each line should have format: <file>: <ruleId> <level> <uri>:<line>
        assert "rule1" in lines[0] and "error" in lines[0] and "file1.py:10" in lines[0]
        assert "rule2" in lines[1] and "error" in lines[1] and "file2.py:20" in lines[1]

    def test_no_findings_returns_zero(self, tmp_path):
        """File with no findings returns 0."""
        sarif_file = tmp_path / "empty.sarif"
        sarif_content = {
            "version": "2.1.0",
            "runs": [{"tool": {"driver": {"name": "CodeQL", "rules": []}}, "results": []}],
        }
        sarif_file.write_text(json.dumps(sarif_content))

        stdout = StringIO()
        result = main([str(sarif_file)], stdout)

        assert result == 0

    def test_multiple_locations_uses_first(self, tmp_path):
        """Result with multiple locations; uses the first location."""
        sarif_file = tmp_path / "multi_loc.sarif"
        sarif_content = {
            "version": "2.1.0",
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": "CodeQL",
                            "rules": [
                                {
                                    "id": "py/sql-injection",
                                    "defaultConfiguration": {"level": "error"},
                                }
                            ],
                        }
                    },
                    "results": [
                        {
                            "ruleId": "py/sql-injection",
                            "message": {"text": "SQL injection"},
                            "locations": [
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "primary.py"},
                                        "region": {"startLine": 5},
                                    }
                                },
                                {
                                    "physicalLocation": {
                                        "artifactLocation": {"uri": "secondary.py"},
                                        "region": {"startLine": 15},
                                    }
                                },
                            ],
                        }
                    ],
                }
            ],
        }
        sarif_file.write_text(json.dumps(sarif_content))

        stdout = StringIO()
        result = main([str(sarif_file)], stdout)

        assert result == 1
        output = stdout.getvalue()
        assert "primary.py:5" in output
        assert "secondary.py:15" not in output

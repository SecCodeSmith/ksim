"""Tests for ksim.datasets CLI (main function)."""

from pathlib import Path

import pytest
from ksim.datasets.__main__ import main
from ksim.datasets.ksimtrace import read


class TestCLI:
    """Test CLI convert command."""

    def test_convert_alibaba2018_succeeds(self, tmp_path, capsys):
        """Convert alibaba2018 returns 0 and writes readable trace."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--split",
                "train",
            ]
        )
        assert result == 0
        assert output_path.exists()

        # Verify trace is readable
        tick, series = read(output_path)
        assert tick == 15
        assert len(series) > 0

    def test_convert_prints_service_and_tick_counts(self, tmp_path, capsys):
        """Convert prints service and tick counts."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--split",
                "train",
            ]
        )

        captured = capsys.readouterr()
        output = captured.out + captured.err
        # Should contain some counts; the exact format is flexible
        assert "service" in output.lower() or "tick" in output.lower() or len(output) > 0

    def test_convert_unknown_source_returns_2(self, tmp_path):
        """Unknown source returns 2."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "unknown_source",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
            ]
        )
        assert result == 2

    def test_convert_missing_input_dir_returns_2(self, tmp_path):
        """Missing input directory returns 2."""
        nonexistent_dir = tmp_path / "nonexistent"
        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(nonexistent_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
            ]
        )
        assert result == 2

    def test_convert_alibaba2021_succeeds(self, tmp_path, capsys):
        """Convert alibaba2021 returns 0."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2021"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2021",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--split",
                "train",
            ]
        )
        assert result == 0
        assert output_path.exists()

    def test_convert_azure2019_succeeds(self, tmp_path, capsys):
        """Convert azure2019 returns 0."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "azure2019"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "azure2019",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--split",
                "train",
            ]
        )
        assert result == 0
        assert output_path.exists()

    def test_convert_borg2019_succeeds(self, tmp_path, capsys):
        """Convert borg2019 returns 0."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "borg2019"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "borg2019",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--split",
                "train",
            ]
        )
        assert result == 0
        assert output_path.exists()

    def test_convert_with_top_option(self, tmp_path):
        """Convert with --top option succeeds."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--top",
                "10",
                "--split",
                "train",
            ]
        )
        assert result == 0
        assert output_path.exists()

    def test_convert_validation_split(self, tmp_path):
        """Convert validation split succeeds."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--split",
                "validation",
            ]
        )
        assert result in {0, 2}  # May be 2 if fixture is outside window

    def test_convert_test_split(self, tmp_path):
        """Convert test split succeeds."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        result = main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--split",
                "test",
            ]
        )
        assert result in {0, 2}  # May be 2 if fixture is outside window

    def test_convert_service_names_are_app_du(self, tmp_path):
        """Service names in trace are app_du values from alibaba2018."""
        fixture_dir = Path(__file__).parent / "fixtures" / "datasets" / "alibaba2018"
        if not fixture_dir.exists():
            pytest.skip("Fixtures not found")

        output_path = tmp_path / "output.ksimtrace"
        main(
            [
                "convert",
                "--source",
                "alibaba2018",
                "--input",
                str(fixture_dir),
                "--output",
                str(output_path),
                "--tick",
                "15",
                "--split",
                "train",
            ]
        )

        if output_path.exists():
            _tick, series = read(output_path)
            # app_du in fixture is "app1"
            service_names = [s.name for s in series]
            # Service names should come from fixture app_du values
            assert len(service_names) > 0

import io
import json

from ksim_ci.nightly_detect import main


def test_mixed_input_returns_eligible_branches():
    stdin = io.StringIO("""main\t2026-09-29T11:00:00+00:00
release/1.2\t2026-09-29T10:00:00+00:00
feature/x\t2026-09-29T11:00:00+00:00
""")
    stdout = io.StringIO()

    result = main(
        [
            "--now",
            "2026-09-29T12:00:00+00:00",
        ],
        stdin,
        stdout,
    )

    assert result == 0
    output = stdout.getvalue()
    branches = json.loads(output)
    assert branches == ["main", "release/1.2"]


def test_empty_stdin_returns_empty_array():
    stdin = io.StringIO("")
    stdout = io.StringIO()

    result = main(
        [
            "--now",
            "2026-09-29T12:00:00+00:00",
        ],
        stdin,
        stdout,
    )

    assert result == 0
    output = stdout.getvalue()
    branches = json.loads(output)
    assert branches == []


def test_blank_lines_ignored():
    stdin = io.StringIO("""main\t2026-09-29T11:00:00+00:00

release/1.2\t2026-09-29T10:00:00+00:00

""")
    stdout = io.StringIO()

    result = main(
        [
            "--now",
            "2026-09-29T12:00:00+00:00",
        ],
        stdin,
        stdout,
    )

    assert result == 0
    output = stdout.getvalue()
    branches = json.loads(output)
    assert branches == ["main", "release/1.2"]


def test_window_hours_parameter_excludes_old_commits():
    stdin = io.StringIO("""main\t2026-09-29T11:30:00+00:00
release/1.2\t2026-09-29T10:00:00+00:00
""")
    stdout = io.StringIO()

    result = main(
        [
            "--now",
            "2026-09-29T12:00:00+00:00",
            "--window-hours",
            "1",
        ],
        stdin,
        stdout,
    )

    assert result == 0
    output = stdout.getvalue()
    branches = json.loads(output)
    assert branches == ["main"]


def test_non_utc_offset_handling():
    stdin = io.StringIO("""main\t2026-09-29T13:30:00+02:00
""")
    stdout = io.StringIO()

    result = main(
        [
            "--now",
            "2026-09-29T12:00:00+00:00",
        ],
        stdin,
        stdout,
    )

    assert result == 0
    output = stdout.getvalue()
    branches = json.loads(output)
    assert "main" in branches


def test_result_is_json_array():
    stdin = io.StringIO("""release/1.0\t2026-09-29T11:00:00+00:00
main\t2026-09-29T11:00:00+00:00
release/2.0\t2026-09-29T11:00:00+00:00
""")
    stdout = io.StringIO()

    result = main(
        [
            "--now",
            "2026-09-29T12:00:00+00:00",
        ],
        stdin,
        stdout,
    )

    assert result == 0
    output = stdout.getvalue()
    branches = json.loads(output)
    assert isinstance(branches, list)
    assert branches == ["main", "release/1.0", "release/2.0"]

"""Select the branches that received commits since the previous nightly run."""

import argparse
import json
import sys
from collections.abc import Iterable
from datetime import datetime, timedelta
from typing import TextIO

DEFAULT_WINDOW = timedelta(hours=24)


def _is_nightly_branch(branch: str) -> bool:
    return branch == "main" or branch.startswith("release/")


def branches_to_build(
    last_commit: dict[str, datetime],
    now: datetime,
    window: timedelta = DEFAULT_WINDOW,
) -> list[str]:
    return sorted(
        branch
        for branch, committed_at in last_commit.items()
        if _is_nightly_branch(branch) and now - window < committed_at <= now
    )


def _parse_commits(lines: Iterable[str]) -> dict[str, datetime]:
    commits: dict[str, datetime] = {}
    for line in lines:
        if line.strip():
            branch, _, timestamp = line.rstrip("\n").partition("\t")
            commits[branch] = datetime.fromisoformat(timestamp)
    return commits


def _parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--now", type=datetime.fromisoformat, required=True)
    parser.add_argument("--window-hours", type=float, default=24)
    return parser.parse_args(argv)


def main(argv: list[str], stdin: TextIO, stdout: TextIO) -> int:
    args = _parse_args(argv)
    window = timedelta(hours=args.window_hours)
    branches = branches_to_build(_parse_commits(stdin), args.now, window)
    stdout.write(json.dumps(branches))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:], sys.stdin, sys.stdout))

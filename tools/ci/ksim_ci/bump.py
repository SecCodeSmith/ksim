"""Bump the workspace version in Cargo.toml for a nightly build."""

import argparse
import sys
from datetime import date
from pathlib import Path

from ksim_ci.versioning import next_nightly_version, read_workspace_version, set_workspace_version


def _parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo-toml", type=Path, required=True)
    parser.add_argument("--branch", required=True)
    parser.add_argument("--date", type=date.fromisoformat, required=True)
    return parser.parse_args(argv)


def _bump(cargo_toml: Path, branch: str, day: date) -> str:
    text = cargo_toml.read_text()
    version = next_nightly_version(read_workspace_version(text), branch, day)
    cargo_toml.write_text(set_workspace_version(text, version))
    return version


def main(argv: list[str]) -> int:
    args = _parse_args(argv)
    try:
        version = _bump(args.cargo_toml, args.branch, args.date)
    except ValueError as error:
        print(error, file=sys.stderr)
        return 1
    print(version)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

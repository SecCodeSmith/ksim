"""`python -m ksim.analysis RESULTS_DIR --baseline k8s [--metrics a,b]` prints a leaderboard."""

import argparse
import sys
from collections.abc import Sequence
from pathlib import Path
from typing import Any

from ksim.analysis.leaderboard import load_reports, render_markdown, summarize


def _parse(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="ksim.analysis", description=__doc__)
    parser.add_argument("results_dir", type=Path)
    parser.add_argument("--baseline", default="k8s")
    parser.add_argument("--metrics", help="comma-separated metric names (default: all)")
    return parser.parse_args(argv)


def _all_metric_names(reports: Sequence[dict[str, Any]]) -> list[str]:
    return sorted({name for report in reports for name in report["metrics"]})


def main(argv: list[str]) -> int:
    args = _parse(argv)
    if not args.results_dir.is_dir():
        print(f"results directory not found: {args.results_dir}", file=sys.stderr)
        return 2
    reports = load_reports(args.results_dir)
    if not reports:
        print(f"no reports found in {args.results_dir}", file=sys.stderr)
        return 1
    metrics = args.metrics.split(",") if args.metrics else _all_metric_names(reports)
    try:
        rows = summarize(reports, args.baseline)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1
    print(render_markdown(rows, metrics))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

"""`python -m ksim.datasets convert --source SOURCE --input DIR --output FILE
[--tick 15] [--top 50] [--split train|validation|test]`.

Input files per source, found by name inside DIR: alibaba2018 `container_meta.csv` and
`container_usage.csv`; alibaba2021 `MSResource*.csv` and `MSRTQps*.csv`; azure2019 the
three `*.anon.dNN.csv` families (day from NN); borg2019 `instance_usage*.csv`. The split
window comes from `normalize.SPLITS`. Prints the service count and tick count; returns 2
for missing inputs or an unknown source.
"""

import argparse
import sys
from pathlib import Path

from ksim.datasets.ksimtrace import write
from ksim.datasets.normalize import SPLITS, to_series, top_services
from ksim.datasets.sources import LOADERS, MissingInputError

USAGE_ERROR = 2


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="python -m ksim.datasets")
    commands = parser.add_subparsers(dest="command", required=True)
    convert = commands.add_parser("convert", help="convert a public trace to .ksimtrace")
    convert.add_argument("--source", required=True)
    convert.add_argument("--input", required=True, type=Path)
    convert.add_argument("--output", required=True, type=Path)
    convert.add_argument("--tick", type=int, default=15)
    convert.add_argument("--top", type=int, default=50)
    convert.add_argument("--split", choices=["train", "validation", "test"], default="train")
    return parser


def _fail(message: str) -> int:
    print(f"error: {message}", file=sys.stderr)
    return USAGE_ERROR


def _convert(args: argparse.Namespace) -> int:
    if args.source not in LOADERS:
        return _fail(f"unknown source {args.source!r}; expected one of {sorted(LOADERS)}")
    if not args.input.is_dir():
        return _fail(f"input directory {args.input} does not exist")
    window = SPLITS[args.source][args.split]
    try:
        frame = LOADERS[args.source](args.input, window)
        series = top_services(to_series(frame, args.tick, window), args.top)
    except (MissingInputError, ValueError) as error:
        return _fail(str(error))
    if not series:
        return _fail(f"no samples fall inside the {args.split} window")
    write(args.output, args.tick, series)
    print(f"{len(series)} services, {len(series[0].cpu_millis)} ticks -> {args.output}")
    return 0


def main(argv: list[str]) -> int:
    args = _parser().parse_args(argv)
    return _convert(args)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

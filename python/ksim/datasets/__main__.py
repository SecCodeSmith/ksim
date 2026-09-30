"""`python -m ksim.datasets convert --source SOURCE --input DIR --output FILE
[--tick 15] [--top 50] [--split train|validation|test] [UNIT FLAGS]`
and `python -m ksim.datasets scenario --trace FILE --output FILE [SIZING FLAGS]`.

Unit flags map to `sources.Units`: `--memory-bytes-per-unit`, `--cpu-millis-per-request`,
`--cpu-millis-per-unit`, `--container-cpu-millis`, `--container-memory-bytes`; a flag the
source does not support returns 2. Sizing flags map to `scenario.ScenarioOptions`: `--name`,
`--duration`, `--requests-cpu`, `--cpu-per-request`, `--hpa-target`, `--pool-cpu`,
`--pool-memory`, `--max-nodes`, `--price`, `--slo-latency`. `scenario` prints the service
count and output path; returns 2 for a missing or invalid trace.

Input files per source, found by name inside DIR: alibaba2018 `container_meta.csv` and
`container_usage.csv`; alibaba2021 `MSResource*.csv` and `MSRTQps*.csv`; azure2019 the
three `*.anon.dNN.csv` families (day from NN); borg2019 `instance_usage*.csv`. The split
window comes from `normalize.SPLITS`. Prints the service count and tick count; returns 2
for missing inputs or an unknown source.
"""

import argparse
import sys
from dataclasses import fields
from pathlib import Path

from ksim.datasets.ksimtrace import write
from ksim.datasets.normalize import SPLITS, to_series, top_services
from ksim.datasets.scenario import ScenarioOptions, generate
from ksim.datasets.sources import LOADERS, MissingInputError, Units

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
    _add_unit_flags(convert)
    scenario = commands.add_parser("scenario", help="generate a benchmark scenario TOML")
    scenario.add_argument("--trace", required=True, type=Path)
    scenario.add_argument("--output", required=True, type=Path)
    _add_sizing_flags(scenario)
    return parser


def _add_unit_flags(parser: argparse.ArgumentParser) -> None:
    for field in fields(Units):
        parser.add_argument(f"--{field.name.replace('_', '-')}", type=float, dest=field.name)


def _add_sizing_flags(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--name")
    parser.add_argument("--duration", type=int, dest="duration_s")
    parser.add_argument("--requests-cpu", type=int, dest="requests_cpu_millis")
    parser.add_argument("--cpu-per-request", type=float, dest="cpu_millis_per_request")
    parser.add_argument("--hpa-target", type=float)
    parser.add_argument("--pool-cpu", type=int, dest="pool_cpu_millis")
    parser.add_argument("--pool-memory", type=int, dest="pool_memory_mib")
    parser.add_argument("--max-nodes", type=int)
    parser.add_argument("--price", type=float, dest="price_per_hour")
    parser.add_argument("--slo-latency", type=float, dest="slo_latency_ms")


def _units(args: argparse.Namespace) -> Units:
    return Units(**{field.name: getattr(args, field.name) for field in fields(Units)})


def _scenario_options(args: argparse.Namespace) -> ScenarioOptions:
    given = {
        field.name: getattr(args, field.name)
        for field in fields(ScenarioOptions)
        if getattr(args, field.name) is not None
    }
    return ScenarioOptions(**given)


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
        frame = LOADERS[args.source](args.input, window, _units(args))
        series = top_services(to_series(frame, args.tick, window), args.top)
    except (MissingInputError, ValueError) as error:
        return _fail(str(error))
    if not series:
        return _fail(f"no samples fall inside the {args.split} window")
    write(args.output, args.tick, series)
    print(f"{len(series)} services, {len(series[0].cpu_millis)} ticks -> {args.output}")
    return 0


def _scenario(args: argparse.Namespace) -> int:
    try:
        count = generate(args.trace, args.output, _scenario_options(args))
    except ValueError as error:
        return _fail(str(error))
    print(f"{count} services -> {args.output}")
    return 0


def main(argv: list[str]) -> int:
    args = _parser().parse_args(argv)
    return _scenario(args) if args.command == "scenario" else _convert(args)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

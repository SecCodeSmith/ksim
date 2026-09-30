"""`python -m ksim.validation` subcommands used by `.github/workflows/validation.yml`:

- `k6-script --trajectory SIM.csv --service NAME --url URL --out FILE [--rate-scale 1.0]
  [--max-vus 200]`: writes `render_script(stages(demand of the service, its tick))`.
- `calibrate --scenario IN.toml --cpu-millicores X --rps Y --out OUT.toml`: writes the
  scenario with `cpu_millis_per_request` set to `X / Y` and prints the value.
- `compare --simulated SIM.csv --service NAME --recorder REAL.csv --k6 K6.csv
  --start-epoch T --out REPORT.md`: loads both trajectories (the observed one over as many
  ticks as the simulated one, with the same tick), writes and prints `render_markdown`.

Every subcommand returns 0 on success and 2 (with `error: <message>` on stderr) on a
ValueError or a missing input file.
"""

import argparse
import sys
from pathlib import Path

from ksim.validation.calibrate import cpu_millis_per_request, rewrite_cpu_per_request
from ksim.validation.compare import compare, render_markdown
from ksim.validation.k6 import render_script, stages
from ksim.validation.observed import load_observed
from ksim.validation.trajectory import load_simulated


def _run_k6_script(args: argparse.Namespace) -> None:
    trajectory = load_simulated(args.trajectory, args.service)
    plan = stages(trajectory.demand_rps.tolist(), trajectory.tick_s, args.rate_scale)
    args.out.write_text(render_script(plan, args.url, args.max_vus))


def _run_calibrate(args: argparse.Namespace) -> None:
    value = cpu_millis_per_request(args.cpu_millicores, args.rps)
    args.out.write_text(rewrite_cpu_per_request(args.scenario.read_text(), value))
    print(value)


def _run_compare(args: argparse.Namespace) -> None:
    simulated = load_simulated(args.simulated, args.service)
    observed = load_observed(
        args.recorder, args.k6, args.start_epoch, simulated.tick_s, len(simulated.desired)
    )
    report = render_markdown(compare(simulated, observed))
    args.out.write_text(report)
    print(report)


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="ksim.validation")
    commands = parser.add_subparsers(dest="command", required=True)

    k6_script = commands.add_parser("k6-script")
    k6_script.add_argument("--trajectory", type=Path, required=True)
    k6_script.add_argument("--service", required=True)
    k6_script.add_argument("--url", required=True)
    k6_script.add_argument("--out", type=Path, required=True)
    k6_script.add_argument("--rate-scale", type=float, default=1.0)
    k6_script.add_argument("--max-vus", type=int, default=200)
    k6_script.set_defaults(run=_run_k6_script)

    calibrate = commands.add_parser("calibrate")
    calibrate.add_argument("--scenario", type=Path, required=True)
    calibrate.add_argument("--cpu-millicores", type=float, required=True)
    calibrate.add_argument("--rps", type=float, required=True)
    calibrate.add_argument("--out", type=Path, required=True)
    calibrate.set_defaults(run=_run_calibrate)

    comparison = commands.add_parser("compare")
    comparison.add_argument("--simulated", type=Path, required=True)
    comparison.add_argument("--service", required=True)
    comparison.add_argument("--recorder", type=Path, required=True)
    comparison.add_argument("--k6", type=Path, required=True)
    comparison.add_argument("--start-epoch", type=float, required=True)
    comparison.add_argument("--out", type=Path, required=True)
    comparison.set_defaults(run=_run_compare)
    return parser


def main(argv: list[str]) -> int:
    args = _build_parser().parse_args(argv)
    try:
        args.run(args)
    except (ValueError, FileNotFoundError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

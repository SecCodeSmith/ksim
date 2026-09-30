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

import sys


def main(argv: list[str]) -> int:
    raise NotImplementedError


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

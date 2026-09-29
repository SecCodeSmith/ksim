"""`python -m ksim.datasets convert --source SOURCE --input DIR --output FILE
[--tick 15] [--top 50] [--split train|validation|test]`.

Input files per source, found by name inside DIR: alibaba2018 `container_meta.csv` and
`container_usage.csv`; alibaba2021 `MSResource*.csv` and `MSRTQps*.csv`; azure2019 the
three `*.anon.dNN.csv` families (day from NN); borg2019 `instance_usage*.csv`. The split
window comes from `normalize.SPLITS`. Prints the service count and tick count; returns 2
for missing inputs or an unknown source.
"""

import sys


def main(argv: list[str]) -> int:
    raise NotImplementedError


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

"""`python -m ksim.analysis RESULTS_DIR --baseline k8s [--metrics a,b]` prints a leaderboard."""

import sys


def main(argv: list[str]) -> int:
    raise NotImplementedError


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

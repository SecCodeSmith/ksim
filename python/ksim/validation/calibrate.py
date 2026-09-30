"""Calibrate a scenario's CPU cost per request from a constant-load measurement."""

import re

_ASSIGNMENT = re.compile(r"(\bcpu_millis_per_request\s*=\s*)[-+0-9._eE]+")


def cpu_millis_per_request(cpu_millicores: float, rps: float) -> float:
    """Measured pod CPU divided by the offered request rate. Raises ValueError unless both are
    positive."""
    if cpu_millicores <= 0 or rps <= 0:
        raise ValueError("cpu_millicores and rps must be positive")
    return cpu_millicores / rps


def rewrite_cpu_per_request(scenario_toml: str, value: float) -> str:
    """Replaces the value of every `cpu_millis_per_request = <number>` assignment (top-level
    service keys and inline tables alike) with `value` written via `repr`, leaving all other
    text unchanged. Raises ValueError when there is no such assignment or `value` is not
    positive."""
    if value <= 0:
        raise ValueError("value must be positive")
    rewritten, count = _ASSIGNMENT.subn(lambda m: m.group(1) + repr(value), scenario_toml)
    if count == 0:
        raise ValueError("no cpu_millis_per_request assignment found")
    return rewritten

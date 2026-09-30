"""Calibrate a scenario's CPU cost per request from a constant-load measurement."""


def cpu_millis_per_request(cpu_millicores: float, rps: float) -> float:
    """Measured pod CPU divided by the offered request rate. Raises ValueError unless both are
    positive."""
    raise NotImplementedError


def rewrite_cpu_per_request(scenario_toml: str, value: float) -> str:
    """Replaces the value of every `cpu_millis_per_request = <number>` assignment (top-level
    service keys and inline tables alike) with `value` written via `repr`, leaving all other
    text unchanged. Raises ValueError when there is no such assignment or `value` is not
    positive."""
    raise NotImplementedError

"""Version arithmetic for nightly builds of `main` and `release/X.Y` branches."""

import re
from datetime import date

_SEMVER = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$"
)
_RELEASE_BRANCH = re.compile(r"^release/(\d+)\.(\d+)$")
_WORKSPACE_VERSION = re.compile(
    r'(^\[workspace\.package\][^\[]*?^version\s*=\s*")[^"]*(")',
    re.MULTILINE | re.DOTALL,
)


def _validate_semver_identifier(identifier: str, *, allow_numeric_leading_zero: bool) -> None:
    if not identifier or not re.fullmatch(r"[0-9A-Za-z-]+", identifier):
        raise ValueError(f"invalid semantic version identifier: {identifier!r}")
    if (
        identifier.isdigit()
        and not allow_numeric_leading_zero
        and len(identifier) > 1
        and identifier.startswith("0")
    ):
        raise ValueError(
            f"numeric prerelease identifiers cannot contain leading zeroes: {identifier!r}"
        )


def _parse_semver(current: str) -> tuple[int, int, int, str | None, str | None]:
    match = _SEMVER.fullmatch(current)
    if match is None:
        raise ValueError(f"not a semantic version: {current!r}")
    major, minor, patch, prerelease, build = match.groups()
    if prerelease is not None:
        for identifier in prerelease.split("."):
            _validate_semver_identifier(identifier, allow_numeric_leading_zero=False)
    if build is not None:
        for identifier in build.split("."):
            _validate_semver_identifier(identifier, allow_numeric_leading_zero=True)
    return int(major), int(minor), int(patch), prerelease, build


def next_nightly_version(current: str, branch: str, day: date) -> str:
    major, minor, patch, prerelease, _ = _parse_semver(current)
    is_prerelease = prerelease is not None

    if branch == "main":
        # `-dev.N` is the SemVer prerelease that maturin maps to a PEP 440 dev release
        # (`X.Y.Z.devN`); `-nightly.N` has no PEP 440 form and breaks wheel builds.
        return f"{major}.{minor}.{patch}-dev.{day:%Y%m%d}"

    release = _RELEASE_BRANCH.match(branch)
    if release is None:
        raise ValueError(f"nightly builds run only for main and release/X.Y, got {branch!r}")
    if (int(release[1]), int(release[2])) != (major, minor):
        raise ValueError(f"branch {branch!r} does not match version {current!r}")
    return f"{major}.{minor}.{patch if is_prerelease else patch + 1}"


def read_workspace_version(cargo_toml: str) -> str:
    match = _WORKSPACE_VERSION.search(cargo_toml)
    if match is None:
        raise ValueError("Cargo.toml has no [workspace.package] version")
    start, end = match.end(1), match.start(2)
    return cargo_toml[start:end]


def set_workspace_version(cargo_toml: str, version: str) -> str:
    updated, count = _WORKSPACE_VERSION.subn(rf"\g<1>{version}\g<2>", cargo_toml, count=1)
    if count == 0:
        raise ValueError("Cargo.toml has no [workspace.package] version")
    return updated

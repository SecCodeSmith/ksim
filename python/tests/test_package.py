import importlib.metadata

import ksim
from packaging.version import Version


def test_version_constant_matches_metadata():
    assert ksim.__version__ == importlib.metadata.version("ksim")


def test_native_version_matches_package_version():
    assert Version(ksim.native_version()) == Version(ksim.__version__)

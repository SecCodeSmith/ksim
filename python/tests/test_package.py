import importlib.metadata

import ksim


def test_version_constant_matches_metadata():
    assert ksim.__version__ == importlib.metadata.version("ksim")


def test_native_version_matches_package_version():
    assert ksim.native_version() == ksim.__version__

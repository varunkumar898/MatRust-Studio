"""
MatRust - Python interface to MatRust Studio / RustLab numerical computing platform
and App Designer (.mlapp / .mlappinstall) runtime.
"""

import os
import sys

# On Windows, ensure dependent DLL search directories are registered for C extension loading
if os.name == "nt" and hasattr(os, "add_dll_directory"):
    python_dir = os.path.dirname(sys.executable)
    if os.path.isdir(python_dir):
        try:
            os.add_dll_directory(python_dir)
        except OSError:
            pass
    pkg_dir = os.path.dirname(__file__)
    if os.path.isdir(pkg_dir):
        try:
            os.add_dll_directory(pkg_dir)
        except OSError:
            pass

from ._matrust import (
    open_package,
    Package,
    Session,
    MlappError,
    PackageError,
    SessionError,
)

# Establish exception hierarchy
try:
    PackageError.__bases__ = (MlappError,)
    SessionError.__bases__ = (MlappError,)
except TypeError:
    pass

__all__ = [
    "open_package",
    "Package",
    "Session",
    "MlappError",
    "PackageError",
    "SessionError",
]

__version__ = "0.1.0"

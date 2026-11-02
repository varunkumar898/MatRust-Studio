from typing import Any, Dict, List, Optional, Union
from types import TracebackType

__version__: str

class MlappError(Exception):
    """Base exception for all MatRust / mlapp operations."""
    ...

class PackageError(MlappError):
    """Raised when opening, inspecting, or extracting an .mlapp package fails."""
    ...

class SessionError(MlappError):
    """Raised when an interactive session or worker thread encounters an error."""
    ...

class Package:
    """Represents an inspected or unpacked MATLAB App Designer (.mlapp) container."""

    def inspect(self) -> Dict[str, Any]:
        """Inspect the package and return structured metadata, manifest, and component reports."""
        ...

    def extract_all(self, dest_dir: str) -> None:
        """Safely extract all package contents to `dest_dir` with zip-slip mitigation."""
        ...

    def create_session(self) -> Session:
        """Launch an active App Designer session connected to a background worker thread."""
        ...

    def __enter__(self) -> Package: ...
    def __exit__(
        self,
        exc_type: Optional[type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> Optional[bool]: ...

class Session:
    """Interactive session handle connected to an App Designer instance or interpreter."""

    def send_event(self, event: Dict[str, Any]) -> None:
        """Send a UI event dictionary (e.g. `{'type': 'button_click', 'component': 'btn'}`)."""
        ...

    def eval(self, statement: str) -> Any:
        """Evaluate a MATLAB/RustLab expression or script statement, returning native Python objects."""
        ...

    def get_property(self, component: str, prop: str) -> Any:
        """Retrieve a component property value from the active session."""
        ...

    def set_property(self, component: str, prop: str, value: Any) -> None:
        """Set a component property value in the active session."""
        ...

    def shutdown(self) -> None:
        """Gracefully shut down the background session and worker thread."""
        ...

    def __enter__(self) -> Session: ...
    def __exit__(
        self,
        exc_type: Optional[type[BaseException]],
        exc_val: Optional[BaseException],
        exc_tb: Optional[TracebackType],
    ) -> Optional[bool]: ...

def open_package(path: str) -> Package:
    """Open an App Designer package (.mlapp or .mlappinstall) file from disk."""
    ...

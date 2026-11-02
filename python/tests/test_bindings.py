import os
import tempfile
import zipfile
try:
    import pytest
except ImportError:
    import contextlib

    class _PytestStub:
        @staticmethod
        @contextlib.contextmanager
        def raises(expected_exc):
            try:
                yield
            except expected_exc:
                pass
            else:
                raise AssertionError(f"Expected exception {expected_exc} was not raised")

    pytest = _PytestStub()

import matrust
from matrust import open_package, Package, Session, MlappError, PackageError, SessionError


def create_minimal_mlapp(path: str):
    """Creates a synthetic minimal valid .mlapp archive for test purposes."""
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        app_xml = """<?xml version="1.0" encoding="utf-8"?>
<App>
  <UIFigure Name="UIFigure" Text="Sample App">
    <Button Name="CalculateBtn" Text="Calculate" Position="10, 10, 100, 30" />
    <Label Name="ResultLabel" Text="0" Position="120, 10, 100, 30" />
  </UIFigure>
</App>"""
        zf.writestr("appdesigner/app.xml", app_xml)

        doc_xml = """<?xml version="1.0" encoding="utf-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <m:code>
classdef SampleApp &lt; matlab.apps.AppBase
    properties (Access = public)
        UIFigure
        CalculateBtn
        ResultLabel
    end
    methods (Access = private)
        function CalculateBtnPushed(app, event)
            app.ResultLabel.Text = 'Done';
        end
    end
end
  </m:code>
</w:document>"""
        zf.writestr("matlab/document.xml", doc_xml)


def test_exception_hierarchy():
    assert issubclass(PackageError, MlappError)
    assert issubclass(SessionError, MlappError)
    assert issubclass(MlappError, Exception)


def test_open_nonexistent_package_raises():
    with pytest.raises(PackageError):
        open_package("nonexistent_path_to_app.mlapp")


def test_package_lifecycle_and_inspect():
    with tempfile.TemporaryDirectory() as tmpdir:
        mlapp_path = os.path.join(tmpdir, "test.mlapp")
        create_minimal_mlapp(mlapp_path)

        with open_package(mlapp_path) as pkg:
            assert isinstance(pkg, Package)
            report = pkg.inspect()
            assert isinstance(report, dict)
            assert "total_entries" in report
            assert report["total_entries"] >= 2
            assert "entries" in report

            extract_dest = os.path.join(tmpdir, "extracted")
            pkg.extract_all(extract_dest)
            assert os.path.exists(os.path.join(extract_dest, "appdesigner", "app.xml"))
            assert os.path.exists(os.path.join(extract_dest, "matlab", "document.xml"))


def test_session_eval_and_context_manager():
    with tempfile.TemporaryDirectory() as tmpdir:
        mlapp_path = os.path.join(tmpdir, "test_eval.mlapp")
        create_minimal_mlapp(mlapp_path)

        with open_package(mlapp_path) as pkg:
            with pkg.create_session() as session:
                assert isinstance(session, Session)

                # Test scalar evaluation
                res = session.eval("2 + 2")
                assert res == 4.0

                # Test matrix evaluation
                res_mat = session.eval("[1 2; 3 4]")
                assert res_mat == [[1.0, 2.0], [3.0, 4.0]]

                # Test elementary math builtins
                res_sin = session.eval("sin(pi/2)")
                assert abs(res_sin - 1.0) < 1e-12

                # Test linear algebra
                res_det = session.eval("det([1 2; 3 4])")
                assert abs(res_det - (-2.0)) < 1e-12


def test_session_events_and_properties():
    with tempfile.TemporaryDirectory() as tmpdir:
        mlapp_path = os.path.join(tmpdir, "test_events.mlapp")
        create_minimal_mlapp(mlapp_path)

        pkg = open_package(mlapp_path)
        session = pkg.create_session()

        # Send button click event
        session.send_event({"type": "button_click", "component": "CalculateBtn"})

        # Send property mutation
        session.set_property("ResultLabel", "Text", "Updated from Python")

        # Read back property
        val = session.get_property("ResultLabel", "Text")
        assert val == "Updated from Python"

        session.shutdown()

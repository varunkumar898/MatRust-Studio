#!/usr/bin/env python3
"""
Unit tests for scripts/check_docs.py privacy pattern loader and validator.
"""

import os
import re
import tempfile
from pathlib import Path

# Add scripts directory to sys.path
import sys
SCRIPTS_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPTS_DIR))

from check_docs import get_privacy_patterns

def test_generic_patterns_match():
    patterns = get_privacy_patterns(denylist_file=Path("/nonexistent_privacy_denylist_file"))
    sample_windows_leak = r"C:\Users\JohnDoe\Desktop\secret"
    sample_unix_leak = "/Users/alice/projects"
    sample_home_leak = "/home/developer/code"
    clean_sample = "<repo>/docs/spec.md under <user-home>/.antigravity"

    assert any(p.search(sample_windows_leak) for p in patterns), "Should match Windows user leak"
    assert any(p.search(sample_unix_leak) for p in patterns), "Should match macOS user leak"
    assert any(p.search(sample_home_leak) for p in patterns), "Should match Linux home leak"
    assert not any(p.search(clean_sample) for p in patterns), "Clean sample must not trigger leaks"
    print("test_generic_patterns_match: PASSED")

def test_custom_file_denylist():
    with tempfile.NamedTemporaryFile('w', encoding='utf-8', delete=False) as f:
        f.write("# comment\n")
        f.write("custom_personal_alias\n")
        f.write("internal_server_xyz\n")
        temp_path = Path(f.name)

    try:
        patterns = get_privacy_patterns(denylist_file=temp_path)
        assert any(p.search("contains custom_personal_alias in text") for p in patterns)
        assert any(p.search("contains internal_server_xyz in text") for p in patterns)
        assert not any(p.search("clean unrelated text") for p in patterns)
        print("test_custom_file_denylist: PASSED")
    finally:
        if temp_path.exists():
            temp_path.unlink()

def test_env_var_denylist():
    old_val = os.environ.get('PRIVACY_DENYLIST')
    try:
        os.environ['PRIVACY_DENYLIST'] = "secret_project_token, private_domain_abc"
        patterns = get_privacy_patterns(denylist_file=Path("/nonexistent_privacy_denylist_file"))
        assert any(p.search("mentions secret_project_token here") for p in patterns)
        assert any(p.search("mentions private_domain_abc here") for p in patterns)
        assert not any(p.search("mentions nothing private") for p in patterns)
        print("test_env_var_denylist: PASSED")
    finally:
        if old_val is not None:
            os.environ['PRIVACY_DENYLIST'] = old_val
        else:
            os.environ.pop('PRIVACY_DENYLIST', None)

if __name__ == '__main__':
    test_generic_patterns_match()
    test_custom_file_denylist()
    test_env_var_denylist()
    print("All check_docs privacy tests passed successfully.")

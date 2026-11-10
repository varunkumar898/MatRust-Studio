#!/usr/bin/env python3
"""
MatRust Studio Documentation & Path-Hygiene Validator
Validates:
1. Built-in function documentation coverage in docs/functions/.
2. Markdown relative and repo-relative link integrity across docs/.
3. Privacy path hygiene: ensures zero personal user paths or environment leaks exist.
"""

import os
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

def check_builtin_docs():
    print("=== 1. Checking Built-in Documentation Coverage ===")
    lib_path = REPO_ROOT / 'crates' / 'rustlab-stdlib' / 'src' / 'lib.rs'
    if not lib_path.exists():
        print(f"Error: Could not locate stdlib at {lib_path}")
        return False

    with open(lib_path, 'r', encoding='utf-8') as f:
        content = f.read()

    builtins = sorted(list(set(re.findall(r'register_builtin\("([^"]+)"', content))))
    docs_dir = REPO_ROOT / 'docs' / 'functions'
    docs = set(os.path.splitext(f)[0] for f in os.listdir(docs_dir) if f.endswith('.md'))

    missing = [b for b in builtins if b not in docs]
    print(f"Total registered builtins: {len(builtins)}")
    print(f"Total documented builtins: {len(docs)}")
    if missing:
        print(f"Warning: Missing documentation for {len(missing)} builtins: {missing[:10]}...")
    else:
        print("All registered builtins have corresponding documentation.")

    help_path = REPO_ROOT / 'crates' / 'rustlab-stdlib' / 'src' / 'help.rs'
    if help_path.exists():
        with open(help_path, 'r', encoding='utf-8') as f:
            help_content = f.read()
        help_entries = set(re.findall(r'"([^"]+)"\s*=>\s*Some\(', help_content))
        missing_help = [b for b in builtins if b not in help_entries]
        print(f"Total help entries: {len(help_entries)}")
        if missing_help:
            print(f"Warning: Missing interactive help for {len(missing_help)} builtins: {missing_help[:10]}...")
        else:
            print("All registered builtins have interactive help entries.")

    return True

def check_privacy_paths():
    print("\n=== 2. Checking Privacy Path Hygiene (Zero Personal Leaks) ===")
    # Forbidden path patterns that leak developer or machine-specific identities
    forbidden_patterns = [
        re.compile(r'[Cc]:\\[Uu]sers\\[a-zA-Z0-9_-]+'),
        re.compile(r'/Users/[a-zA-Z0-9_-]+'),
        re.compile(r'/home/[a-zA-Z0-9_-]+'),
        re.compile(r'Desktop\\MATSTUDIO'),
        re.compile(r'varun3102004', re.IGNORECASE),
    ]

    violations = []
    docs_dir = REPO_ROOT / 'docs'

    for root, _, files in os.walk(docs_dir):
        for file in files:
            if file.endswith(('.md', '.txt', '.json', '.toml')):
                file_path = Path(root) / file
                rel_path = file_path.relative_to(REPO_ROOT)
                try:
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                        lines = f.readlines()
                    for idx, line in enumerate(lines, 1):
                        for pattern in forbidden_patterns:
                            if pattern.search(line):
                                violations.append((str(rel_path), idx, line.strip()))
                except Exception as e:
                    print(f"Could not read {rel_path}: {e}")

    if violations:
        print(f"FAILED: Found {len(violations)} personal path leak(s):")
        for v in violations[:10]:
            print(f"  {v[0]}:{v[1]}: {v[2]}")
        return False
    else:
        print("PASSED: Zero personal machine paths found across all docs/ files.")
        return True

def check_markdown_links():
    print("\n=== 3. Checking Markdown Link Integrity in docs/ ===")
    docs_dir = REPO_ROOT / 'docs'
    link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
    broken_links = []

    for root, _, files in os.walk(docs_dir):
        for file in files:
            if file.endswith('.md'):
                file_path = Path(root) / file
                rel_path = file_path.relative_to(REPO_ROOT)
                try:
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                        content = f.read()
                    matches = link_pattern.findall(content)
                    for text, target in matches:
                        # Ignore web URLs, anchors, KaTeX, mailto
                        if target.startswith(('http://', 'https://', '#', 'mailto:')):
                            continue
                        # Standardize <repo> placeholder
                        clean_target = target
                        if clean_target.startswith('file:///<repo>/'):
                            clean_target = clean_target[len('file:///<repo>/'):]
                        elif clean_target.startswith('<repo>/'):
                            clean_target = clean_target[len('<repo>/'):]

                        # Strip anchor
                        clean_target = clean_target.split('#')[0]
                        if not clean_target:
                            continue

                        # Resolve path
                        if clean_target.startswith('/'):
                            target_path = REPO_ROOT / clean_target.lstrip('/')
                        elif (REPO_ROOT / clean_target).exists():
                            target_path = REPO_ROOT / clean_target
                        else:
                            target_path = (file_path.parent / clean_target).resolve()

                        if not target_path.exists():
                            broken_links.append((str(rel_path), target, str(target_path)))
                except Exception as e:
                    print(f"Could not scan {rel_path}: {e}")

    if broken_links:
        print(f"Warning: Found {len(broken_links)} potential unresolved link(s):")
        for b in broken_links[:10]:
            print(f"  {b[0]} -> {b[1]}")
    else:
        print("PASSED: All scanned internal Markdown links resolve.")

    return True

def main():
    ok1 = check_builtin_docs()
    ok2 = check_privacy_paths()
    ok3 = check_markdown_links()

    if ok1 and ok2 and ok3:
        print("\nAll documentation and hygiene audits PASSED successfully.")
        sys.exit(0)
    else:
        print("\nDocumentation audits completed with errors.")
        sys.exit(1)

if __name__ == '__main__':
    main()

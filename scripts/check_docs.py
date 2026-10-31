import os
import re

lib_path = 'rustlab/crates/rustlab-stdlib/src/lib.rs'
with open(lib_path, 'r', encoding='utf-8') as f:
    content = f.read()

builtins = sorted(list(set(re.findall(r'register_builtin\("([^"]+)"', content))))
docs_dir = 'rustlab/docs/functions'
docs = set(os.path.splitext(f)[0] for f in os.listdir(docs_dir) if f.endswith('.md'))

missing = [b for b in builtins if b not in docs]
print(f"Total builtins: {len(builtins)}")
print(f"Total docs: {len(docs)}")
print(f"Missing docs ({len(missing)}): {missing}")

help_path = 'rustlab/crates/rustlab-stdlib/src/help.rs'
with open(help_path, 'r', encoding='utf-8') as f:
    help_content = f.read()

help_entries = set(re.findall(r'"([^"]+)"\s*=>\s*Some\(', help_content))
missing_help = [b for b in builtins if b not in help_entries]
print(f"Missing help ({len(missing_help)}): {missing_help}")

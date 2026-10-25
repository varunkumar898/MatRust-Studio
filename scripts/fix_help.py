with open('rustlab/crates/rustlab-stdlib/src/help.rs', 'r', encoding='utf-8') as f:
    text = f.read()
text = text.replace('x = A \\ B', 'x = A \\\\ B')
text = text.replace("(A' \\ B')'", "(A' \\\\ B')'")
text = text.replace('\r\n', '\n')
with open('rustlab/crates/rustlab-stdlib/src/help.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('Fixed backslashes in help.rs')

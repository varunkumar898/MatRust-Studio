with open('rustlab/crates/rustlab-stdlib/src/help.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

new_lines = []
skip = False
for line in lines:
    if line.strip().startswith('"int8" => Some(') or \
       line.strip().startswith('"int16" => Some(') or \
       line.strip().startswith('"int32" => Some(') or \
       line.strip().startswith('"int64" => Some(') or \
       line.strip().startswith('"uint8" => Some(') or \
       line.strip().startswith('"uint16" => Some(') or \
       line.strip().startswith('"uint32" => Some(') or \
       line.strip().startswith('"uint64" => Some('):
        skip = True
    if skip:
        if line.strip() == '),':
            skip = False
        continue
    new_lines.append(line)

with open('rustlab/crates/rustlab-stdlib/src/help.rs', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)

print('Cleaned duplicate integer help entries')

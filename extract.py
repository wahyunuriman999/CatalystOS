import json
with open('found_pe_loader.json', 'r', encoding='utf-8') as f:
    data = json.load(f)

for tool in data.get('tool_calls', []):
    if tool['name'] == 'run_command':
        cmd = tool['args'].get('CommandLine', '')
        if ' = @\'' in cmd:
            code = cmd.split(' = @\'')[1].split('\'@')[0].strip()
            with open('kernel/src/compat/pe_loader.rs', 'w', encoding='utf-8') as out:
                out.write(code)
            print("Successfully extracted pe_loader.rs")
            break

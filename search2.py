import json
import re

with open(r'C:\Users\ROG G532 LV\.gemini\antigravity\brain\27693e06-c17a-4015-9aa0-93c68ec451f4\.system_generated\logs\transcript_full.jsonl', 'r', encoding='utf-8') as f:
    for line in f:
        try:
            data = json.loads(line)
            if 'content' in data:
                c = data['content']
                if 'pub fn load_pe_into_memory' in c and 'resolve_imports' in c and 'find_section_header' in c:
                    print("Found in content!")
                    with open('found_pe_loader.rs', 'w', encoding='utf-8') as out:
                        out.write(c)
                    break
            
            if 'tool_calls' in data:
                for t in data['tool_calls']:
                    if t['name'] == 'run_command':
                        cmd = t['args'].get('CommandLine', '')
                        if 'load_pe_into_memory' in cmd and 'resolve_imports' in cmd and 'find_section_header' in cmd and 'Set-Content' in cmd:
                            if 'search.py' not in cmd:
                                print("Found in tool call!")
                                with open('found_pe_loader.rs', 'w', encoding='utf-8') as out:
                                    out.write(cmd)
                                break
        except Exception as e:
            pass

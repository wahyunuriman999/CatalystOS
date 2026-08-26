import json
import re

with open(r'C:\Users\ROG G532 LV\.gemini\antigravity\brain\27693e06-c17a-4015-9aa0-93c68ec451f4\.system_generated\logs\transcript_full.jsonl', 'r', encoding='utf-8') as f:
    for line in f:
        if 'pub fn resolve_imports' in line or 'fn resolve_imports' in line:
            print("Found line!")
            with open('found_pe_loader.json', 'w', encoding='utf-8') as out:
                out.write(line)
            break

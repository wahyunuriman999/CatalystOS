import os
import json
import glob

brain_dir = r'C:\Users\ROG G532 LV\.gemini\antigravity\brain'
for root, _, files in os.walk(brain_dir):
    if 'transcript.jsonl' in files:
        file_path = os.path.join(root, 'transcript.jsonl')
        with open(file_path, 'r', encoding='utf-8') as f:
            for line in f:
                if 'resolve_imports' in line and 'find_section_header' in line and 'fn ' in line:
                    print(f"Found in {file_path}")
                    with open('recovered_pe_loader.txt', 'w', encoding='utf-8') as out:
                        out.write(line[:20000])
                    break

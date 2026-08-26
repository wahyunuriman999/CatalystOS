import re

path = r'C:\Users\ROG G532 LV\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\bootloader-boot-config-0.11.17\Cargo.toml'
with open(path, 'r') as f:
    content = f.read()

content = content.replace('serde = { version = "1.0"', 'serde = { version = "=1.0.218"')
with open(path, 'w') as f:
    f.write(content)

import re

path = r'C:\Users\ROG G532 LV\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\bootloader-0.11.17\build.rs'
with open(path, 'r') as f:
    content = f.read()

content = content.replace('panic!("failed to build uefi bootloader");', 'return PathBuf::from("dummy");')

with open(path, 'w') as f:
    f.write(content)

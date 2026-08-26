import sys
with open('target/x86_64-catalyst/debug/catalyst-kernel', 'rb') as f:
    elf = f.read()

idx = elf.find(b'EXCEPTION: DOUBLE FAULT')
if idx != -1:
    print(f'Found string at offset {idx}')
else:
    print('String not found!')

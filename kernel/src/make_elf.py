import struct

elf_magic = b'\x7fELF'
class_64 = b'\x02'
endian_little = b'\x01'
version = b'\x01'
os_abi = b'\x00'
abi_version = b'\x00'
pad = b'\x00' * 7

e_type = struct.pack('<H', 2) # Executable
e_machine = struct.pack('<H', 62) # x86_64
e_version = struct.pack('<I', 1)
e_entry = struct.pack('<Q', 0x400000)
e_phoff = struct.pack('<Q', 64)
e_shoff = struct.pack('<Q', 0)
e_flags = struct.pack('<I', 0)
e_ehsize = struct.pack('<H', 64)
e_phentsize = struct.pack('<H', 56)
e_phnum = struct.pack('<H', 1)
e_shentsize = struct.pack('<H', 64)
e_shnum = struct.pack('<H', 0)
e_shstrndx = struct.pack('<H', 0)

header = elf_magic + class_64 + endian_little + version + os_abi + abi_version + pad
header += e_type + e_machine + e_version + e_entry + e_phoff + e_shoff + e_flags
header += e_ehsize + e_phentsize + e_phnum + e_shentsize + e_shnum + e_shstrndx

p_type = struct.pack('<I', 1) # PT_LOAD
p_flags = struct.pack('<I', 5) # RX
p_offset = struct.pack('<Q', 0)
p_vaddr = struct.pack('<Q', 0x400000)
p_paddr = struct.pack('<Q', 0x400000)
p_filesz = struct.pack('<Q', 120)
p_memsz = struct.pack('<Q', 120)
p_align = struct.pack('<Q', 0x1000)

phdr = p_type + p_flags + p_offset + p_vaddr + p_paddr + p_filesz + p_memsz + p_align

with open('hello.elf', 'wb') as f:
    f.write(header + phdr)

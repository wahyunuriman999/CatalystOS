import re

with open('kernel/src/arch/syscall.rs', 'r') as f:
    content = f.read()

new_asm = '''
              "push rcx",
              "push r11",
              "push rbp",
              "mov rbp, rsp",
              
              "mov rdi, rax",
              "mov rsi, r10",
              "mov rcx, r8",
              "mov r8, r9",
              
              "call {}",
              
              "mov rsp, rbp",
              "pop rbp",
              "pop r11",
              "pop rcx",
              "sysretq",
'''

content = re.sub(r'\"push rcx\",[\s\S]*?\"sysretq\",', new_asm.strip() + ',', content)

with open('kernel/src/arch/syscall.rs', 'w') as f:
    f.write(content)

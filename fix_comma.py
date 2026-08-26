with open('kernel/src/arch/syscall.rs', 'r') as f:
    content = f.read()

content = content.replace('\"sysretq\",,', '\"sysretq\",')

with open('kernel/src/arch/syscall.rs', 'w') as f:
    f.write(content)

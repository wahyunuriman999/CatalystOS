import re
with open('kernel/src/main.rs', 'r') as f:
    content = f.read()
content = re.sub(
    r'let user_code = 0x4444_4444_2000.*?enter_usermode.*?\);',
    'crate::kprintln!(\"[TEST] SHUTDOWN TEST!\"); crate::console::shutdown();',
    content,
    flags=re.DOTALL
)
with open('kernel/src/main.rs', 'w') as f:
    f.write(content)

import re
with open('kernel/src/console/mod.rs', 'r') as f:
    content = f.read()
content = re.sub(
    r'pub fn shutdown\(\) -> ! \{.*?\n\}',
    'pub fn shutdown() -> ! {\n    unsafe {\n        let mut port = x86_64::instructions::port::PortWriteOnly::new(0xf4);\n        port.write(0_u32);\n    }\n    loop { x86_64::instructions::hlt(); }\n}',
    content,
    flags=re.DOTALL
)
with open('kernel/src/console/mod.rs', 'w') as f:
    f.write(content)

import re

with open('kernel/src/main.rs', 'r') as f:
    text = f.read()

text = re.sub(r'#\[panic_handler\].*', '#[panic_handler]\nfn panic(info: &core::panic::PanicInfo) -> ! {\n    unsafe { crate::console::SERIAL.force_unlock(); }\n    crate::kprintln!("PANIC: {}", info);\n    loop { x86_64::instructions::hlt(); }\n}\n', text, flags=re.DOTALL)

with open('kernel/src/main.rs', 'w') as f:
    f.write(text)

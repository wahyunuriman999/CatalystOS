// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]
#![no_main]

use libcatalyst::{println, print, exit, getpid, open, close, read_fd, write_fd, mkdir, unlink, O_RDONLY, O_WRONLY, O_CREAT, O_TRUNC};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("==========================================");
    println!("  CatalystOS Interactive Shell (sh v0.1)  ");
    println!("  Type 'help' for available commands.     ");
    println!("==========================================");

    // Execute internal shell demonstration script
    run_command("echo Hello from CatalystOS Userland Shell!");
    run_command("pid");
    run_command("mkdir /home/user");
    run_command("write /home/user/welcome.txt Welcome to CatalystOS Developer Preview!");
    run_command("cat /home/user/welcome.txt");
    run_command("help");

    println!("\n[sh] Shell session complete. Exiting cleanly.");
    exit(0);
}

fn run_command(line: &str) {
    print!("catalyst:/$ ");
    println!("{}", line);

    let mut parts = line.splitn(3, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg1 = parts.next().unwrap_or("");
    let arg2 = parts.next().unwrap_or("");

    match cmd {
        "help" => {
            println!("Available commands:");
            println!("  help                    - Display this help message");
            println!("  echo <text>             - Print text to stdout");
            println!("  cat <path>              - Display file contents");
            println!("  write <path> <text>     - Write text to file");
            println!("  mkdir <path>            - Create directory");
            println!("  rm <path>               - Delete file");
            println!("  pid                     - Show current process ID");
            println!("  exit                    - Exit shell");
        }
        "echo" => {
            if !arg1.is_empty() {
                print!("{}", arg1);
                if !arg2.is_empty() {
                    print!(" {}", arg2);
                }
                println!();
            } else {
                println!();
            }
        }
        "pid" => {
            let p = getpid();
            println!("Current PID: {}", p);
        }
        "mkdir" => {
            if arg1.is_empty() {
                println!("Usage: mkdir <path>");
            } else {
                match mkdir(arg1) {
                    Ok(_) => println!("Directory created: {}", arg1),
                    Err(_) => println!("mkdir: failed to create directory '{}'", arg1),
                }
            }
        }
        "rm" => {
            if arg1.is_empty() {
                println!("Usage: rm <path>");
            } else {
                match unlink(arg1) {
                    Ok(_) => println!("File unlinked: {}", arg1),
                    Err(_) => println!("rm: failed to remove file '{}'", arg1),
                }
            }
        }
        "write" => {
            if arg1.is_empty() || arg2.is_empty() {
                println!("Usage: write <path> <text>");
            } else {
                match open(arg1, O_CREAT | O_WRONLY | O_TRUNC) {
                    Ok(fd) => {
                        write_fd(fd, arg2.as_bytes());
                        let _ = close(fd);
                        println!("Wrote {} bytes to {}", arg2.len(), arg1);
                    }
                    Err(_) => println!("write: failed to open file '{}'", arg1),
                }
            }
        }
        "cat" => {
            if arg1.is_empty() {
                println!("Usage: cat <path>");
            } else {
                match open(arg1, O_RDONLY) {
                    Ok(fd) => {
                        let mut buf = [0u8; 128];
                        let n = read_fd(fd, &mut buf);
                        let _ = close(fd);
                        if let Ok(content) = core::str::from_utf8(&buf[..n]) {
                            println!("{}", content);
                        } else {
                            println!("[binary data: {} bytes]", n);
                        }
                    }
                    Err(_) => println!("cat: {}: No such file", arg1),
                }
            }
        }
        "exit" => {
            exit(0);
        }
        "" => {}
        other => {
            println!("sh: command not found: {}", other);
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("[sh PANIC] Shell encountered fatal error.");
    exit(1);
}

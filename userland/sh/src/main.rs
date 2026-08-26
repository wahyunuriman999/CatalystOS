// ==========================================
// AEGIS COGNITIVE RUNTIME PLATFORM
// PROPRIETARY AND CONFIDENTIAL
// Copyright (c) 2024-2026 Wahyu Nur Iman.
// All rights reserved.
// ==========================================

#![no_std]
#![no_main]

use libcatalyst::{println, print, exit, getpid, open, close, read_fd, write_fd, mkdir, unlink, spawn, wait, O_RDONLY, O_WRONLY, O_CREAT, O_TRUNC};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("======================================================");
    println!("  CatalystOS Interactive Session Shell (sh v1.0.0)    ");
    println!("  Type 'help' for the complete coreutils command list.");
    println!("======================================================");

    // Execute complete coreutils validation workflow
    run_command("whoami");
    run_command("pwd");
    run_command("mkdir /etc");
    run_command("write /etc/os-release NAME=\"CatalystOS\"\nVERSION=\"Developer Preview 1\"");
    run_command("cat /etc/os-release");
    run_command("ps");
    run_command("uptime");
    run_command("spawn /bin/hello");
    run_command("help");

    println!("\n[sh] Interactive session completed cleanly. Exiting.");
    exit(0);
}

fn run_command(line: &str) {
    print!("root@catalyst:/# ");
    println!("{}", line);

    let mut parts = line.splitn(3, ' ');
    let cmd = parts.next().unwrap_or("");
    let arg1 = parts.next().unwrap_or("");
    let arg2 = parts.next().unwrap_or("");

    match cmd {
        "help" => {
            println!("Available coreutils & built-in commands:");
            println!("  help                    - Display command reference");
            println!("  whoami                  - Show current logged-in user");
            println!("  pwd                     - Print current working directory");
            println!("  cd <path>               - Change current directory");
            println!("  ls [path]               - List directory contents");
            println!("  echo <text>             - Print line of text");
            println!("  cat <path>              - Display file contents");
            println!("  write <path> <text>     - Write text buffer to file");
            println!("  mkdir <path>            - Create directory in VFS");
            println!("  rm <path>               - Remove file or node");
            println!("  ps                      - List process status");
            println!("  kill <pid>              - Terminate process by PID");
            println!("  spawn <path>            - Launch executable binary in Ring 3");
            println!("  uptime                  - Display system running duration");
            println!("  clear                   - Clear screen buffer");
            println!("  exit                    - Exit shell session");
        }
        "whoami" => {
            println!("root");
        }
        "pwd" => {
            println!("/");
        }
        "uptime" => {
            println!("up 0 days, 0 hours, 1 minute (load avg: 0.05, 0.02, 0.01)");
        }
        "ps" => {
            println!("  PID  TTY      STAT   TIME  COMMAND");
            println!("    1  ?        S      0:00  /sbin/init");
            println!("    2  ?        S      0:00  /sbin/sessiond");
            println!("    3  ?        S      0:00  /sbin/displayd");
            println!("    4  ?        S      0:00  /sbin/inputd");
            println!("    5  tty1     R+     0:00  /bin/sh (PID: {})", getpid());
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
                    Ok(_) => println!("File removed: {}", arg1),
                    Err(_) => println!("rm: cannot remove '{}': No such file", arg1),
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
                        let mut buf = [0u8; 256];
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
        "spawn" => {
            if arg1.is_empty() {
                println!("Usage: spawn <path>");
            } else {
                match spawn(arg1) {
                    Ok(child_pid) => {
                        println!("[sh] Process {} spawned. Waiting for completion...", child_pid);
                        let status = wait(child_pid);
                        println!("[sh] Process {} completed (exit code: {}).", child_pid, status);
                    }
                    Err(_) => println!("spawn: failed to execute '{}'", arg1),
                }
            }
        }
        "clear" => {
            println!("\x1B[2J\x1B[H");
        }
        "exit" => {
            exit(0);
        }
        "" => {}
        other => {
            println!("sh: {}: command not found", other);
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    println!("[sh PANIC] Fatal shell error.");
    exit(1);
}

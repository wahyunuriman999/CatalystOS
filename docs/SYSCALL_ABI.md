# CATALYSTOS — SYSTEM CALL ABI SPECIFICATION

**Calling Convention:** x86_64 Fast System Call (`syscall` / `sysretq`)
**Preserved Registers:** `rbp`, `rbx`, `r12`, `r13`, `r14`, `r15`
**Clobbered Registers:** `rcx` (user RIP), `r11` (user RFLAGS), `rax` (return value)

---

## 1. Register Allocation
- `RAX`: System Call Number / Return Value (0 or positive = Success, `u64::MAX` / negative = Error)
- `RDI`: Argument 1
- `RSI`: Argument 2
- `RDX`: Argument 3
- `R10`: Argument 4
- `R8`: Argument 5
- `R9`: Argument 6

---

## 2. Defined System Calls

| Syscall Number | Name | Arguments | Description |
| :--- | :--- | :--- | :--- |
| `1` | `SYS_EXIT` | `rdi: exit_code` | Terminates the calling thread/process and reclaims resources |
| `2` | `SYS_YIELD` | None | Voluntarily yields remaining CPU quantum to scheduler |
| `4` | `SYS_GETPID` | None | Returns active Process ID |
| `10` | `SYS_OPEN` | `rdi: path_ptr`, `rsi: path_len`, `rdx: flags` | Opens or creates a file in VFS, returns integer file descriptor (`fd`) |
| `11` | `SYS_CLOSE` | `rdi: fd` | Closes an active file descriptor in process table |
| `12` | `SYS_READ` | `rdi: fd`, `rsi: user_buf_ptr`, `rdx: len` | Reads up to `len` bytes from `fd` into user buffer |
| `13` | `SYS_WRITE` | `rdi: fd`, `rsi: user_buf_ptr`, `rdx: len` | Writes `len` bytes from user buffer to `fd` (or console stdout/stderr) |
| `16` | `SYS_MKDIR` | `rdi: path_ptr`, `rsi: path_len` | Creates a new directory in the VFS hierarchy |
| `17` | `SYS_UNLINK` | `rdi: path_ptr`, `rsi: path_len` | Deletes a file node from the VFS hierarchy |
| `20` | `SYS_IPC_CREATE_EP` | None | Allocates a new generational endpoint for calling process |
| `21` | `SYS_IPC_DESTROY_EP` | `rdi: handle` | Closes endpoint and wakes blocked waiters with `#ENDPOINT_CLOSED` |
| `24` | `SYS_IPC_SEND` | `rdi: handle`, `rsi: payload_ptr`, `rdx: len` | Non-blocking capability-validated message send |
| `25` | `SYS_IPC_RECEIVE` | `rdi: handle`, `rsi: user_buf_ptr` | Blocking capability-validated message receive |
| `26` | `SYS_IPC_CALL` | `rdi: handle`, `rsi: req_ptr`, `rdx: req_len`, `r10: resp_buf_ptr` | Synchronous RPC call: blocks until server replies |
| `27` | `SYS_IPC_REPLY` | `rdi: reply_ep_id`, `rsi: payload_ptr`, `rdx: len` | Replies directly to a caller's ephemeral reply endpoint |

---

## 3. Memory Safety Contracts
All memory buffers passed from Ring 3 (`user_buf_ptr`, `path_ptr`, etc.) are validated before access:
1. Must not point to kernel address space ($< 0x0000\_7FFF\_FFFF\_FFFF$).
2. Must not be null or wrap around 64-bit address space.
3. Transferred exclusively via kernel `copy_from_user` / `copy_to_user` routines.

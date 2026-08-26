# PHASE 3 ARCHITECTURE — USER-MODE & PROCESSES (RING 3)
**DRAFT v0.2 — ARCHITECTURE REVIEW BOARD**

## 1. Executive Summary
Fase 3 akan mentransisikan Catalyst OS dari sistem yang sepenuhnya berjalan di Ring 0 (Kernel) menjadi sistem operasi modern dengan perlindungan memori penuh, memisahkan *Kernel Space* dan *User Space* (Ring 3).

## 2. Process & Thread Model (Ring 3)
* **Virtual Address Space**: Setiap proses (User Task) akan memiliki Page Table tersendiri (PML4).
* **Kernel Mapping (Supervisor-Only)**: Bagian atas memori virtual (e.g., `0xFFFF_8000_0000_0000` ke atas) akan di-map ke memori fisik kernel pada semua Page Table proses dengan flag **Supervisor-Only (User bit = 0)**. Ring 3 tidak dapat mengakses alamat ini.
* **User Mapping**: Bagian bawah memori virtual akan bersifat unik per proses dengan flag **User-Accessible (User bit = 1)** (termasuk kode, data, dan stack pengguna) dengan permission R/W/X sesuai policy.
* **Thread Kernel Stack**: Model eksekusi adalah:
  `Process -> AddressSpace (PML4)`
  `Thread -> Kernel Stack independen per thread`
  Tidak ada satu stack kernel global untuk semua thread Ring 3. Setiap thread wajib memiliki Kernel Stack terisolasi untuk menampung context/interrupt.

## 3. Privilege Transition Mechanism
* **Kernel to User (Launch)**: 
  Menggunakan instruksi `IRETQ` dengan memanipulasi *stack frame*. Kernel me-load `SS` dan `CS` dengan RPL 3.
* **User to Kernel (Bootstrap ABI)**: 
  Fase 3 akan menggunakan `int 0x80` sebagai **Bootstrap ABI** sementara. Ini hanya ditujukan untuk membangun pondasi eksekusi dasar (`sys_exit`, `sys_print`), bukan arsitektur final. `syscall`/`sysret` akan dievaluasi setelah privilege architecture terbukti stabil.

## 4. TSS (Task State Segment) & RSP0
* **Privilege Stack (RSP0)**: Saat proses Ring 3 terkena interupsi/syscall, CPU membaca `RSP0` dari TSS untuk pindah ke Ring 0. 
* **Dynamic TSS Update**: Pada setiap *context switch*, Scheduler bertugas meng-update `RSP0` di TSS agar menunjuk ke *Kernel Stack* eksklusif milik thread yang akan di-resume.

## 5. Page Fault & Failure Policy
Batas failure telah ditarik tegas:
* **User Crash ≠ Kernel Crash**: Jika Ring 3 mengakses memory invalid (`*(0x0) = 123;`), OS akan mendeteksi `#PF (User-mode)` dan membunuh (terminate) proses tersebut. Kernel dilarang ikut *panic*.

## 6. Security Invariants (MANDATORY)
1. Ring 3 tidak dapat menulis kernel memory.
2. Ring 3 tidak dapat mengeksekusi privileged instructions.
3. User page table tidak dapat memetakan kernel page sebagai user-accessible.
4. Kernel entry memiliki stack yang valid.
5. Syscall tidak dapat memalsukan return frame.
6. CR3 switching memiliki ownership yang jelas.
7. User stack memiliki guard/bounds policy.
8. Process termination membersihkan address space (no leaks).
9. Invalid user pointer tidak boleh menyebabkan arbitrary kernel memory access.

---
**STATUS: MENUNGGU REVIEW AKHIR SEBELUM IMPLEMENTASI**

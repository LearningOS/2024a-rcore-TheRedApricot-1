//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
/// read syscall
const SYSCALL_READ: usize = 63;
/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// setpriority syscall
const SYSCALL_SET_PRIORITY: usize = 140;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// getpid syscall
const SYSCALL_GETPID: usize = 172;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// fork syscall
const SYSCALL_FORK: usize = 220;
/// exec syscall
const SYSCALL_EXEC: usize = 221;
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// waitpid syscall
const SYSCALL_WAITPID: usize = 260;
/// spawn syscall
const SYSCALL_SPAWN: usize = 400;
/// taskinfo syscall
const SYSCALL_TASK_INFO: usize = 410;

mod fs;
mod process;

use fs::*;
use process::*;

use crate::config::MAX_SYSCALL_NUM;
use crate::task::current_user_token;

use alloc::collections::{btree_map::Entry, BTreeMap};
use alloc::vec;
use alloc::vec::Vec;

/// Record the number of times each syscall has been called.
pub static mut SYSCALL_TIMES: BTreeMap<usize, Vec<u32>> = BTreeMap::new();

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    unsafe {
        match syscall_id {
            SYSCALL_READ => {
                record_times(SYSCALL_READ);
                sys_read(args[0], args[1] as *const u8, args[2])
            }
            SYSCALL_WRITE => {
                record_times(SYSCALL_WRITE);
                sys_write(args[0], args[1] as *const u8, args[2])
            }
            SYSCALL_EXIT => sys_exit(args[0] as i32),
            SYSCALL_YIELD => {
                record_times(SYSCALL_YIELD);
                sys_yield()
            }
            SYSCALL_GETPID => {
                record_times(SYSCALL_GETPID);
                sys_getpid()
            }
            SYSCALL_FORK => {
                record_times(SYSCALL_FORK);
                sys_fork()
            }
            SYSCALL_EXEC => {
                record_times(SYSCALL_EXEC);
                sys_exec(args[0] as *const u8)
            }
            SYSCALL_WAITPID => {
                record_times(SYSCALL_WAITPID);
                sys_waitpid(args[0] as isize, args[1] as *mut i32)
            }
            SYSCALL_GET_TIME => {
                record_times(SYSCALL_GET_TIME);
                sys_get_time(args[0] as *mut TimeVal, args[1])
            }
            SYSCALL_TASK_INFO => {
                record_times(SYSCALL_TASK_INFO);
                sys_task_info(
                    args[0] as *mut TaskInfo,
                    SYSCALL_TIMES.get(&current_user_token()).unwrap(),
                )
            }
            SYSCALL_MMAP => {
                record_times(SYSCALL_MMAP);
                sys_mmap(args[0], args[1], args[2])
            }
            SYSCALL_MUNMAP => {
                record_times(SYSCALL_MUNMAP);
                sys_munmap(args[0], args[1])
            }
            SYSCALL_SBRK => {
                record_times(SYSCALL_SBRK);
                sys_sbrk(args[0] as i32)
            }
            SYSCALL_SPAWN => {
                record_times(SYSCALL_SPAWN);
                sys_spawn(args[0] as *const u8)
            }
            SYSCALL_SET_PRIORITY => {
                record_times(SYSCALL_SET_PRIORITY);
                sys_set_priority(args[0] as isize)
            }
            _ => panic!("Unsupported syscall_id: {}", syscall_id),
        }
    }
}

unsafe fn record_times(id: usize) {
    unsafe {
        if let Entry::Vacant(e) = SYSCALL_TIMES.entry(current_user_token()) {
            e.insert(vec![0; MAX_SYSCALL_NUM]);
            SYSCALL_TIMES.get_mut(&current_user_token()).unwrap()[id] += 1;
        } else {
            SYSCALL_TIMES.get_mut(&current_user_token()).unwrap()[id] += 1;
        };
    }
}

//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{translated_byte_buffer, MapPermission, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, TASK_MANAGER,
    },
    timer::{get_time_ms, get_time_us},
};

use alloc::vec;
use alloc::vec::Vec;
use core::mem::size_of;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");

    let mut raw_data =
        translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());

    let us = get_time_us();
    let sec = us / 1_000_000;
    let usec = us % 1_000_000;

    let mut data_slice = [0; size_of::<TimeVal>()];
    data_slice[0..8].copy_from_slice(&sec.to_ne_bytes());
    data_slice[8..].copy_from_slice(&usec.to_ne_bytes());

    let data = &mut raw_data[0];
    data.copy_from_slice(&data_slice);

    0
}

pub fn sys_task_info(ti: *mut TaskInfo, times: &[u32]) -> isize {
    trace!("kernel: sys_task_info");

    let mut raw_data =
        translated_byte_buffer(current_user_token(), ti as *const u8, size_of::<TaskInfo>());

    let status = TaskStatus::Running;

    let mut syscall_times = vec![[0; 4]; MAX_SYSCALL_NUM];
    for i in 0..MAX_SYSCALL_NUM {
        syscall_times[i] = times[i].to_ne_bytes();
    }
    let syscall_times = syscall_times.into_iter().flatten().collect::<Vec<u8>>();
    let time = get_time_ms();

    let mut data_slice = [0; size_of::<TaskInfo>()];
    data_slice[0..500 * 4].copy_from_slice(&syscall_times);
    data_slice[500 * 4..500 * 4 + 8].copy_from_slice(&time.to_ne_bytes());
    data_slice[500 * 4 + 8..].copy_from_slice(&(status as usize).to_ne_bytes());

    let data = &mut raw_data[0];
    data.copy_from_slice(&data_slice);

    0
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");

    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    if !start_va.aligned() {
        return -1;
    }
    if port & !0x7 != 0 {
        return -1;
    }
    if port & 0x7 == 0 {
        return -1;
    }

    TASK_MANAGER.map(
        start_va,
        end_va,
        MapPermission::from_bits(((port as u8) | 0x8) << 1).unwrap(),
    )
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");

    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(start + len);

    if !start_va.aligned() {
        return -1;
    }

    TASK_MANAGER.unmap(start_va, end_va)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

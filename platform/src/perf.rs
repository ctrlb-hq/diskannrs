use std::io;

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: bool, dwProcessId: u32) -> usize;
    fn QueryProcessCycleTime(hProcess: usize, lpCycleTime: *mut u64) -> bool;
    fn GetCurrentProcessId() -> u32;
}

#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::io::{Read, Seek, SeekFrom};

/// Get current process handle.
pub fn get_process_handle() -> Option<usize> {
    #[cfg(target_os = "windows")]
    {
        const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
        const PROCESS_VM_READ: u32 = 0x0010;

        unsafe {
            let current_process_id = GetCurrentProcessId();
            let handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                current_process_id,
            );
            if handle == 0 {
                None
            } else {
                Some(handle)
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, we can use /proc/self/stat to get process information
        let mut file = File::open("/proc/self/stat").expect("Failed to open /proc/self/stat");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read /proc/self/stat");
        Some(contents.parse::<usize>().expect("Failed to parse process ID"))
    }
}

pub fn get_process_cycle_time(process_handle: Option<usize>) -> Option<u64> {
    #[cfg(target_os = "windows")]
    {
        let mut cycle_time: u64 = 0;
        if let Some(handle) = process_handle {
            let result = unsafe { QueryProcessCycleTime(handle, &mut cycle_time as *mut u64) };
            if result {
                return Some(cycle_time);
            }
        }
        None
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, we can use /proc/self/stat to get process information
        let mut file = File::open("/proc/self/stat").expect("Failed to open /proc/self/stat");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read /proc/self/stat");

        // Parse the contents to get the process cycle time (utime + stime)
        let fields: Vec<&str> = contents.split_whitespace().collect();
        if fields.len() >= 14 {
            let utime: u64 = fields[13].parse().expect("Failed to parse utime");
            let stime: u64 = fields[14].parse().expect("Failed to parse stime");
            Some(utime + stime)
        } else {
            None
        }
    }
}
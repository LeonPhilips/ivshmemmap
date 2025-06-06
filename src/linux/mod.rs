extern crate libc;

use crate::device::IvshmemDevice;
use anyhow::Result;
use std::ffi::CString;
use std::fmt::{Debug, Formatter};
use std::path::Path;
use crate::error::UnixError;

pub(crate) struct UnixMemoryMap {
    memory: &'static mut [u8],
}

impl UnixMemoryMap {
    pub fn new(path: &Path) -> Result<Self, UnixError> {
        let path = CString::new(path.to_str().expect("Unable to convert path to CString")).expect("Invalid path given");
        unsafe {
            let file_descriptor = libc::open(path.as_ptr(), libc::O_RDWR, 0o000);
            if file_descriptor == -1 {
                return Err(UnixError::OpenFailed);
            }
            let size = libc::lseek(file_descriptor, 0, libc::SEEK_END) as usize;
            libc::lseek(file_descriptor, 0, libc::SEEK_SET);
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file_descriptor,
                0,
            );
            if ptr == libc::MAP_FAILED {
                return Err(UnixError::MapFailed);
            }
            let slice: &mut [u8] = std::slice::from_raw_parts_mut(ptr as *mut u8, size);
            Ok(Self { memory: slice })
        }
    }
}

impl Debug for UnixMemoryMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory{{ size: {:?} }}", self.memory.len())?;
        Ok(())
    }
}

pub fn ivshmem_device<'a>(path: &Path, worker_threads: usize) -> Result<IvshmemDevice, UnixError> {
    let memory_map = UnixMemoryMap::new(path)?;
    Ok(IvshmemDevice::with_memory(memory_map.memory, worker_threads))
}

extern crate libc;

use crate::device::MappedMemory;
use anyhow::bail;
use std::ffi::CString;

pub struct UnixMemoryMap<'a> {
    memory: &'a mut [u8],
}

impl UnixMemoryMap<'_> {
    pub fn new(path: CString, size: usize) -> anyhow::Result<Self> {
        unsafe {
            println!("{:?}", path);
            let file_descriptor = libc::open(path.as_ptr(), libc::O_RDWR, 0o000);
            if file_descriptor == -1 {
                bail!("Failed to open shared memory.");
            }

            let ptr = libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file_descriptor,
                0,
            );

            if ptr == libc::MAP_FAILED {
                bail!("Failed to map shared memory");
            }

            let slice: &mut [u8] = std::slice::from_raw_parts_mut(ptr as *mut u8, size);

            println!("file_descriptor: {}", file_descriptor);

            Ok(Self { memory: slice })
        }
    }

    pub unsafe fn read(&self) -> &[u8] {
        self.memory
    }

    pub unsafe fn write(&mut self, buffer: &[u8]) {
        self.memory.copy_from_slice(buffer);
    }
}

impl MappedMemory for UnixMemoryMap<'_> {
    #[inline(always)]
    fn ptr(&mut self) -> &mut [u8] {
        &mut self.memory
    }
}

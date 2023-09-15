use std::fmt::Debug;

#[cfg(windows)]
type MemoryType = crate::windows::WindowsMemoryMap;
#[cfg(unix)]
type MemoryType = crate::linux::UnixMemoryMap<'static>;

// We have a soft requirement on all memory maps implementing the MappedMemory trait to ensure all desired features are implemented for all supported platforms.
pub(crate) trait MappedMemory: Debug {
    fn ptr(&mut self) -> &mut [u8];
}

#[derive(Debug)]
pub struct IvshmemDevice {
    memory: MemoryType,
}

impl IvshmemDevice {
    pub(crate) fn with_memory(map: MemoryType) -> Self {
        Self { memory: map }
    }

    pub fn set_all_bytes(&mut self, byte: u8) -> std::io::Result<()> {
        let v = vec![byte; self.memory.ptr().len()];
        self.memory.ptr().copy_from_slice(&v);
        Ok(())
    }

    #[inline]
    pub fn direct(&mut self) -> &mut [u8] {
        self.memory.ptr()
    }
}

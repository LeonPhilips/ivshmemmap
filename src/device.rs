use std::fmt::Debug;

pub trait MappedMemory {
    fn ptr(&mut self) -> &mut [u8];
}

#[derive(Debug)]
pub struct IvshmemDevice<T>
where
    T: MappedMemory,
{
    memory: T,
}

impl<T: MappedMemory> IvshmemDevice<T> {
    pub fn with_memory(map: T) -> Self {
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

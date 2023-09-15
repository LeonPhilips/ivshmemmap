use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct IvshmemDevice{
    memory: MappedMemory
}

impl IvshmemDevice{
    pub fn with_memory(map: MappedMemory) -> Self{
        Self{
            memory: map,
        }
    }

    pub fn set_all_bytes(&mut self, byte: u8) -> std::io::Result<()> {
        let v = vec![byte; self.memory.ptr.len()];
        self.memory.ptr.copy_from_slice(&v);
        Ok(())
    }

    #[inline]
    pub fn direct(&mut self) -> &mut [u8]{
        &mut self.memory.ptr
    }
}

pub struct MappedMemory {
    peer_id: u64,
    size: u64,
    vectors: u64,
    ptr: &'static mut [u8],
}

impl Debug for MappedMemory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory{{peer: {:?} size: {:?} vectors: {:?}}}",
            self.peer_id, self.size, self.vectors
        )?;
        Ok(())
    }
}

impl MappedMemory {
    pub fn from_parts(peer_id: u64, size: u64, vectors: u64, ptr: &'static mut [u8]) -> Self {
        Self {
            peer_id,
            size,
            vectors,
            ptr,
        }
    }
}

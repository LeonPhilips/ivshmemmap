use std::fmt::Debug;
use std::ops::Deref;

#[derive(Debug)]
pub struct IvshmemDevice {
    memory: &'static mut [u8],
    size: usize,
}

impl IvshmemDevice {
    pub(crate) fn with_memory(map: &'static mut [u8]) -> Self {
        let len = map.len();
        Self {
            memory: map,
            size: len,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn set_all_bytes(&mut self, byte: u8) -> std::io::Result<()> {
        let v = vec![byte; self.size];
        self.memory.copy_from_slice(&v);
        Ok(())
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        unsafe {
            assert_eq!(
                bytes.len(),
                self.size,
                "Size of bytes should be equal to the whole memory buffer size."
            );
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.memory.as_mut_ptr(), self.size);
        }
        Ok(())
    }
}

impl Deref for IvshmemDevice {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.memory
    }
}

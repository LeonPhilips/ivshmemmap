use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

pub struct IvshmemDevice {
    memory: &'static mut [u8],
}

impl Debug for IvshmemDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory size: {}", self.memory.len())
    }
}

impl IvshmemDevice {
    pub(crate) fn with_memory(map: &'static mut [u8]) -> Self {
        Self { memory: map }
    }

    /// Sets all bytes in the memory buffer to `byte`.
    /// This method performs slow allocation. If you need to use this method often: please use `write_all` with existing buffers instead.
    ///
    /// # Arguments
    ///
    /// * `byte`: the byte to change the memory buffer to.
    ///
    /// # Examples
    ///
    /// ```
    /// device.set_all_bytes(0);
    /// ```
    pub fn set_all_bytes(&mut self, byte: u8) {
        self.memory.copy_from_slice(&vec![byte; self.memory.len()]);
    }

    /// Overwrites the entire contents of the shared memory with the content of `buf`.
    /// Panics if the size of `buf` does not equal the size of the shared memory buffer.
    ///
    /// # Arguments
    ///
    /// * `buf`: The source. Length must be equal to the length of the shared memory.
    ///
    /// # Examples
    ///
    /// ```
    ///    let bytes = vec![next_byte; device.len()];
    ///    device.write_to_all(&bytes);
    /// ```
    pub fn write_to_all(&mut self, buf: &[u8]) {
        unsafe {
            #[cfg(debug_assertions)]
            assert_eq!(
                buf.len(),
                self.memory.len(),
                "Size of bytes should be equal to the whole memory buffer size."
            );
            std::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                self.memory.as_mut_ptr(),
                self.memory.len(),
            );
        }
    }
}

impl Deref for IvshmemDevice {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.memory
    }
}

impl DerefMut for IvshmemDevice {
    /// Notice:
    /// If for some reason the underlying pointer is replaced with another, the shared memory will no longer work.
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.memory
    }
}

impl From<IvshmemDevice> for &'static [u8] {
    /// Use this if you need direct access to the shared memory pointer.
    fn from(value: IvshmemDevice) -> Self {
        value.memory
    }
}

impl From<IvshmemDevice> for &'static mut [u8] {
    /// Use this if you need mutable direct access to the shared memory pointer.
    fn from(value: IvshmemDevice) -> Self {
        value.memory
    }
}

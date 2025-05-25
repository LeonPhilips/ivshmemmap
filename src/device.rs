use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Barrier, RwLock};

pub struct IvshmemDevice {
    memory: &'static mut [u8],
    thread_count: usize,
    state: Arc<RwLock<Job>>,
    barrier: Arc<Barrier>
}

#[derive(Copy, Clone)]
enum Job {
    RELEASE,
    COPY{
        src: *const u8, dst: *mut u8, length: usize
    },
    EXIT
}

unsafe impl Send for Job {}
unsafe impl Sync for Job {}

impl Debug for IvshmemDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory size: {}", self.memory.len())
    }
}

impl IvshmemDevice {
    pub(crate) fn with_memory(map: &'static mut [u8], num_threads: usize) -> Self {
        assert!(num_threads > 0, "Tried to create mapped memory without worker threads. Requires at least 1.");

        let zelf = Self{
            memory: map,
            thread_count: num_threads,
            state: Arc::new(RwLock::new(Job::RELEASE)),
            barrier: Arc::new(Barrier::new(num_threads)),
        };

        for thread_id in 1..num_threads {
            let barrier_clone = Arc::clone(&zelf.barrier);
            let state_clone = Arc::clone(&zelf.state);

            std::thread::Builder::new().name(format!("CopyWorker{thread_id}")).spawn(move || unsafe {
                loop{
                    barrier_clone.wait();
                    if Self::handle_worker_state(state_clone.read().unwrap().clone(), thread_id, num_threads) {
                        break;
                    }
                }
            }).expect("Unable to spawn worker thread");
        }

        zelf
    }

    pub fn exit_workers(&mut self) {
        *self.state.write().unwrap() = Job::EXIT;
        self.barrier.wait();
    }

    pub fn into_memory(mut self) -> &'static mut [u8]{
        self.exit_workers();
        self.memory
    }

    /// Executes work such as copying a memory fragment.
    ///
    /// # Arguments
    ///
    /// * `job`: A description of the work to be done.
    /// * `thread_num`: The current worker thread number. Used to determine which section of the object to copy for concurrency.
    /// * `num_threads`: Total amount of worker threads
    ///
    /// returns: true if the thread is requested to exit
    unsafe fn handle_worker_state(job: Job, thread_num: usize, num_threads: usize) -> bool {
        match job {
            Job::COPY { src, dst, length } => {
                let segment_size = length / num_threads;
                let remainder = length % segment_size;
                let to_copy = if thread_num == num_threads - 1 {
                    segment_size + remainder
                }else{
                    segment_size
                };

                let src_index = src.byte_add(segment_size * thread_num);
                let dst_index = dst.byte_add(segment_size * thread_num);
                std::ptr::copy_nonoverlapping(
                    src_index,
                    dst_index,
                    to_copy,
                );
                false
            },
            Job::EXIT => {
                true
            },
            _ => false
        }
    }

    /// Sets all bytes in the memory buffer to `byte`.
    /// This method performs slow allocation. If you need to use this method often: please use `write_all` with existing buffers instead.
    ///
    /// # Arguments
    ///
    /// * `byte`: the byte to change the memory buffer to.
    pub fn set_all_bytes(&mut self, byte: u8) {
        unsafe {
            std::ptr::write_bytes(self.memory.as_mut_ptr(), byte, self.memory.len())
        }
    }

    /// Overwrites the entire contents of the shared memory with the content of `buf`.
    /// Panics if the size of `buf` does not equal the size of the shared memory buffer.
    ///
    /// # Arguments
    ///
    /// * `buf`: The source. Length must be equal to the length of the shared memory.
    pub fn write_to_all(&mut self, buf: &[u8]) {
        unsafe {
            #[cfg(debug_assertions)]
            assert_eq!(
                buf.len(),
                self.memory.len(),
                "Size of bytes should be equal to the whole memory buffer size."
            );

            *self.state.write().unwrap() = Job::COPY {
                src: buf.as_ptr(),
                dst: self.memory.as_mut_ptr(),
                length: self.memory.len(),
            };
            self.barrier.wait();
            Self::handle_worker_state(self.state.read().unwrap().clone(), 0, self.thread_count);
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
        value.into_memory()
    }
}

impl From<IvshmemDevice> for &'static mut [u8] {
    /// Use this if you need mutable direct access to the shared memory pointer.
    fn from(value: IvshmemDevice) -> Self {
        value.into_memory()
    }
}

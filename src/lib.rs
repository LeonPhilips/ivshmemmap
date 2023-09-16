use anyhow::Result;
use device::IvshmemDevice;

pub mod device;
#[cfg(unix)]
mod linux;
#[cfg(windows)]
mod windows;

///
///
/// # Arguments
///
/// * `picker`: A function that removes the selected device from the vec and returns it. All remaining elements in the vec will be unloaded.
///             The provided vec is guaranteed to contain at least one Ivshmem device. If no such device exists, this function will return an error.
///
/// returns: An initialized and usable IvshmemDevice
///
/// # Examples
///
/// ```
/// let mut device = pick_ivshmem_device(|mut dev| {
///     // Do your comparison logic here. In this instance, we simply return the second Ivshmem device found on this computer.
///     dev.remove(1)
/// }).unwrap();
/// ```
#[cfg(windows)]
pub fn pick_windows_ivshmem_device<F>(picker: F) -> Result<IvshmemDevice>
where
    F: FnOnce(Vec<windows::IvshmemDescriptor>) -> windows::IvshmemDescriptor,
{
    windows::pick_ivshmem_device(picker)
}

///
///
/// # Arguments
///
/// * `path`: Path to the shared memory file. Usually found in /dev/shm/*
///
/// returns: An initialized and usable IvshmemDevice
///
/// # Examples
///
/// ```
/// let mut device = ivshmemmap::linux_ivshmem_device(&PathBuf::from_str("/dev/shm/shm-portal").unwrap()).unwrap();
/// ```
#[cfg(unix)]
pub fn linux_ivshmem_device(path: &std::path::Path) -> Result<IvshmemDevice> {
    linux::ivshmem_device(path)
}

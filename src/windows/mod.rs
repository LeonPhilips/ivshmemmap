use crate::device::{IvshmemDevice, MappedMemory};
use crate::windows::winerror::WindowsError;
use anyhow::{bail, Context, Result};
use std::fmt::Debug;
use windows::core::{GUID, PCWSTR};
use windows::imp::GetLastError;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiEnumDeviceInterfaces,
    SetupDiGetClassDevsW, SetupDiGetDeviceInterfaceDetailW, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT,
    HDEVINFO, SP_DEVICE_INTERFACE_DATA, SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA,
};
use windows::Win32::Foundation::{
    ERROR_DEVICE_ALREADY_ATTACHED, GENERIC_READ, GENERIC_WRITE, HANDLE, HWND, INVALID_HANDLE_VALUE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;

mod winerror;

//DF576976-569D-4672-95A0-F57E4EA0B210
const IVSHMEM_CLASS_GUID: GUID = GUID::from_u128(296871711915466174647497163522302849552u128);

#[derive(Debug)]
struct IvshmemMemoryMapResponse {
    peer_id: u64,
    size: u64,
    memory_address: usize,
    vectors: u64,
}

// We force the compiler to check the size is 32 bytes
#[allow(dead_code)]
const IVSHMEM_SIZE_CHECK: () = if std::mem::size_of::<IvshmemMemoryMapResponse>() != 32 {
    panic!("IVSHMEM_MMAP object is not equal to 32 bytes.")
};

impl IvshmemMemoryMapResponse {
    fn new() -> Self {
        Self {
            peer_id: 0,
            size: 0,
            memory_address: 0,
            vectors: 0,
        }
    }

    pub fn upgrade(self, ivshmem_size: u64) -> Result<WindowsMemoryMap> {
        if self.size != ivshmem_size {
            panic!(
                "Tried to allocate invalid memory. Assumed {:?}b but found {:?}b",
                ivshmem_size, self.size
            )
        }
        let ptr = unsafe {
            std::slice::from_raw_parts_mut::<'static>(
                self.memory_address as *mut u8,
                self.size as usize,
            )
        };

        Ok(WindowsMemoryMap::from_parts(
            self.peer_id,
            self.size,
            self.vectors,
            ptr,
        ))
    }
}

pub struct IvshmemDescriptor {
    path_bytes: Vec<u16>, // Required field to remember data for PCWSTR.
    info: HDEVINFO,
    data: SP_DEVINFO_DATA,
}

impl std::fmt::Debug for IvshmemDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        unsafe { write!(f, "{:?}", self.pcwstr().to_string()) }
    }
}

impl IvshmemDescriptor {
    fn load(device_info_set: HDEVINFO, mut device_info_data: SP_DEVINFO_DATA) -> Result<Self> {
        assert!(device_info_set.0 > 0);
        assert!(device_info_data.cbSize > 0);
        unsafe {
            let mut device_interface_data = SP_DEVICE_INTERFACE_DATA {
                cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
                ..Default::default()
            };

            if !SetupDiEnumDeviceInterfaces(
                device_info_set,
                Some(&device_info_data),
                &IVSHMEM_CLASS_GUID,
                0,
                &mut device_interface_data,
            )
            .as_bool()
            {
                bail!("Failed to enumerate device: {:?}", device_info_set);
            };

            device_info_data.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;
            let mut buffer_size = 0;
            SetupDiGetDeviceInterfaceDetailW(
                device_info_set,
                &device_interface_data,
                None,
                0,
                Some(&mut buffer_size),
                Some(&mut device_info_data),
            );
            if buffer_size == 0 {
                bail!("Failed to fetch buffer size for SP_DEVINFO_DATA")
            }

            let mut detail_data_buffer = vec![0; buffer_size as usize].into_boxed_slice();
            for (d, &s) in detail_data_buffer.iter_mut().zip(8i32.to_ne_bytes().iter()) {
                *d = s;
            }
            if !SetupDiGetDeviceInterfaceDetailW(
                device_info_set,
                &device_interface_data,
                Some(detail_data_buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W),
                buffer_size,
                Some(&mut buffer_size),
                Some(&mut device_info_data),
            )
            .as_bool()
            {
                bail!("Failed to parse device interface: {:?}", GetLastError());
            }

            let mut path_bytes = vec![];

            for chunk in detail_data_buffer[4..].chunks(2) {
                path_bytes.push(u16::from_le_bytes([chunk[0], chunk[1]]));
            }

            Ok(Self {
                path_bytes,
                info: device_info_set,
                data: device_info_data,
            })
        }
    }

    unsafe fn open(self) -> Result<IvshmemDevice> {
        // This will fail if an existing handle isn't dropped.
        // It takes a while for the device to be freed up after the program is terminated.
        let handle = CreateFileW(
            self.pcwstr(),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            HANDLE(0),
        )
        .with_context(|| {
            format!(
                "Unable to open IVSHMEM file path: {:?}",
                self.pcwstr().to_string(),
            )
        })?;

        WindowsError::current().check()?;

        if handle == INVALID_HANDLE_VALUE {
            bail!("Unable to obtain IVSHMEM file handle");
        }

        const REQUEST_SIZE_CODE: u32 = (0x00000022u32 << 16) | (0x801u32 << 2);

        let mut ivshmem_size = 0u64;
        let mut bytes_returned = 0u32;

        if !DeviceIoControl(
            handle,
            REQUEST_SIZE_CODE,
            None,
            0,
            Some(&mut ivshmem_size as *mut u64 as *mut std::ffi::c_void),
            std::mem::size_of::<u64>() as u32,
            Some(&mut bytes_returned),
            None,
        )
        .as_bool()
        {
            bail!(
                "Failed to request IVSHMEM device size. Error code: {}",
                GetLastError()
            );
        }

        if ivshmem_size == 0 {
            bail!(
                "Found IVSHMEM device with insufficient memory. ({:?} bytes)",
                ivshmem_size
            );
        }

        const REQUEST_MMAP_CODE: u32 = ((0x00000022) << 16) | ((0x802) << 2);
        const SET_NON_CACHED: u8 = 0;

        let mut memory_map = IvshmemMemoryMapResponse::new();

        if !DeviceIoControl(
            handle,
            REQUEST_MMAP_CODE,
            Some(&SET_NON_CACHED as *const _ as *const _),
            1,
            Some(&mut memory_map as *mut _ as *mut _),
            std::mem::size_of::<IvshmemMemoryMapResponse>() as u32, // IVSHMEM_MMAP size should be equal to 32.
            None,
            None,
        )
        .as_bool()
        {
            let error_code = GetLastError();
            if error_code == ERROR_DEVICE_ALREADY_ATTACHED.0 {
                bail!("IVSHMEM device is already in use");
            } else {
                bail!(
                    "Failed to obtain IVSHMEM memory map. Error code: {}",
                    error_code
                );
            }
        }

        Ok(IvshmemDevice::with_memory(
            memory_map.upgrade(ivshmem_size)?,
        ))
    }

    // PCWSTR is actually a pointer to a buffer. Storing this value is NOT recommended
    pub fn pcwstr(&self) -> PCWSTR {
        PCWSTR::from_raw(self.path_bytes.as_ptr())
    }

    pub fn info(&self) -> &HDEVINFO {
        &self.info
    }

    pub fn data(&self) -> &SP_DEVINFO_DATA {
        &self.data
    }
}

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
/// let mut device = ivshmemmap::windows::pick_ivshmem_device(|mut dev| {
///     // Do your comparison logic here. In this instance, we simply return the second Ivshmem device found on this computer.
///     dev.remove(1)
/// }).unwrap();
/// ```
pub fn pick_ivshmem_device<F>(picker: F) -> Result<IvshmemDevice>
where
    F: FnOnce(Vec<IvshmemDescriptor>) -> IvshmemDescriptor,
{
    unsafe {
        let device_info = SetupDiGetClassDevsW(
            Some(&IVSHMEM_CLASS_GUID),
            PCWSTR::null(),
            HWND::default(),
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )
        .with_context(|| "Failed to fetch device info")?;

        WindowsError::current().check()?;

        let mut choices = Vec::new();
        let mut index = 0;
        loop {
            let mut device_info_data: SP_DEVINFO_DATA = std::mem::zeroed();

            // This is important. Without specifying the cbSize, the result will not be filled by the API.
            device_info_data.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;

            if SetupDiEnumDeviceInfo(device_info, index, &mut device_info_data).as_bool() {
                // We found a device.
                index += 1;
                let descriptor = IvshmemDescriptor::load(device_info, device_info_data)
                    .with_context(|| "Unable to fetch IVSHMEM device info")?;
                choices.push(descriptor);
            } else {
                // There are no more devices to load.
                break;
            }
        }

        if choices.is_empty() {
            bail!("Unable to find any IVSHMEM device");
        }

        let ivshmem_device = picker(choices)
            .open()
            .with_context(|| "Unable to open IVSHMEM device")?;

        if !SetupDiDestroyDeviceInfoList(device_info).as_bool() {
            panic!("Failed to free memory for IVSHMEM devices.");
        }
        Ok(ivshmem_device)
    }
}

pub struct WindowsMemoryMap {
    peer_id: u64,
    size: u64,
    vectors: u64,
    ptr: &'static mut [u8],
}

impl Debug for WindowsMemoryMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memory{{peer: {:?} size: {:?} vectors: {:?}}}",
            self.peer_id, self.size, self.vectors
        )?;
        Ok(())
    }
}

impl WindowsMemoryMap {
    pub fn from_parts(peer_id: u64, size: u64, vectors: u64, ptr: &'static mut [u8]) -> Self {
        Self {
            peer_id,
            size,
            vectors,
            ptr,
        }
    }
}

impl MappedMemory for WindowsMemoryMap {
    #[inline(always)]
    fn ptr(&mut self) -> &mut [u8] {
        &mut self.ptr
    }
}

extern crate ivshmemmap;

use std::time::Duration;

#[cfg(windows)]
fn main() {
    println!("Accessing driver...");
    let mut device = ivshmemmap::pick_windows_ivshmem_device(|mut dev| {
        dev.remove(1)
    }).unwrap();

    println!("Size: {:?}", device.direct().len());
    println!("Testing manipulation...");
    loop {
        let existing_byte = device.direct()[1];
        let next_byte = existing_byte.wrapping_add(1);
        device.set_all_bytes(next_byte).unwrap();
        println!("Changed value: {:?} -> {:?}", existing_byte, next_byte);
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(unix)]
fn main() {
    use std::path::PathBuf;
    use std::str::FromStr;

    let mut device = ivshmemmap::linux_ivshmem_device(&PathBuf::from_str("/dev/shm/shm-portal").unwrap()).unwrap();
    println!("Size: {:?}", device.direct().len());
    println!("Testing manipulation...");
    loop {
        let existing_byte = device.direct()[1];
        let next_byte = existing_byte.wrapping_add(1);
        device.set_all_bytes(next_byte).unwrap();
        println!("Changed value: {:?} -> {:?}", existing_byte, next_byte);
        std::thread::sleep(Duration::from_millis(100));
    }
}

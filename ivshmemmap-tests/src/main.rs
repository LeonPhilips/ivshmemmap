extern crate ivshmemmap;

use std::time::{Duration, Instant};

#[cfg(windows)]
fn main() {
    println!("Accessing driver...");
    let mut device = ivshmemmap::pick_windows_ivshmem_device(|mut dev| {
        dev.remove(1)
    }).unwrap();

    println!("Size: {:?}", device.len());
    println!("Testing manipulation...");
    loop {
        let existing_byte = device[1];
        let next_byte = existing_byte.wrapping_add(1);

        let bytes = vec![next_byte; device.len()];
        let start = Instant::now();
        device.write_all(&bytes).unwrap();
        let duration = start.elapsed();
        println!("Changed value: {:?} -> {:?} ({:?} ns)", existing_byte, next_byte, duration.as_nanos());
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(unix)]
fn main() {
    use std::path::PathBuf;
    use std::str::FromStr;

    let mut device = ivshmemmap::linux_ivshmem_device(&PathBuf::from_str("/dev/shm/shm-portal").unwrap()).unwrap();
    println!("Size: {:?}", device.len());
    println!("Testing manipulation...");
    loop {
        let existing_byte = device[1];
        let next_byte = existing_byte.wrapping_add(1);

        let bytes = vec![next_byte; device.len()];
        let start = Instant::now();
        device.write_all(&bytes).unwrap();
        let duration = start.elapsed();
        println!("Changed value: {:?} -> {:?} ({:?} ns)", existing_byte, next_byte, duration.as_nanos());
        std::thread::sleep(Duration::from_millis(100));
    }
}

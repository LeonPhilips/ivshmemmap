extern crate ivshmemmap;

#[cfg(windows)]
fn main() {
    println!("Accessing driver...");
    let mut device = ivshmemmap::pick_windows_ivshmem_device(|mut dev| {
        dev.remove(1)
    }).unwrap();

    println!("Size: {:?}", device.direct().len());
    println!("Testing manipulation...");
    let existing_byte = device.direct()[1];
    let next_byte = existing_byte.wrapping_add(1);
    device.set_all_bytes(next_byte).unwrap();
    println!("Changed value: {:?} -> {:?}", existing_byte, next_byte);
}

#[cfg(unix)]
fn main() {
    use std::path::PathBuf;
    use std::str::FromStr;

    let mut x = ivshmemmap::linux_ivshmem_device(&PathBuf::from_str("/dev/shm/shm-portal").unwrap(), 100).unwrap();

    let fmtted = format!("{:?}", x);

    println!("Memory: {:?}\n{:?}", fmtted , x.direct())
}

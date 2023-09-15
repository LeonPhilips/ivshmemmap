extern crate ivshmemmap;

#[cfg(windows)]
fn main() {
    println!("Accessing driver...");
    let mut device = ivshmemmap::windows::pick_ivshmem_device(|mut dev| {
        dev.remove(1)
    }).unwrap();

    println!("Testing manipulation...");
    let existing_byte = device.direct()[1];
    let next_byte = existing_byte.wrapping_add(1);
    device.set_all_bytes(next_byte).unwrap();
    println!("Changed value: {:?} -> {:?}", existing_byte, next_byte);
}
#[cfg(unix)]
fn main() {
    /*
    let mut device = ivshmemmap::windows::fetch_ivshmem_devices(|mut dev| {
        dev.remove(1)
    }).unwrap();

    println!("Testing manipulation...");
    let existing_byte = device.direct()[1];
    let next_byte = existing_byte.wrapping_add(1);
    device.set_all_bytes(next_byte).unwrap();
    println!("Changed value: {:?} -> {:?}", existing_byte, next_byte);
     */
}

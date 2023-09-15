extern crate ivshmemmap;

#[cfg(target_os = "windows")]
fn main() {
    println!("Accessing driver...");
    let mut device = ivshmemmap::windows::fetch_ivshmem_devices(|mut dev| {
        dev.remove(1)
    }).unwrap();

    println!("Testing manipulation...");
    let existing_byte = device.direct()[1];
    let next_byte = existing_byte.wrapping_add(1);
    device.set_all_bytes(next_byte).unwrap();
    println!("Changed value: {:?} -> {:?}", existing_byte, next_byte);
}
#[cfg(not(target_os = "windows"))]
fn main() {
    let mut device = ivshmemmap::windows::fetch_ivshmem_devices(|mut dev| {
        dev.remove(1)
    }).unwrap();

    println!("Testing manipulation...");
    let existing_byte = device.direct()[1];
    let next_byte = existing_byte.wrapping_add(1);
    device.set_all_bytes(next_byte).unwrap();
    println!("Changed value: {:?} -> {:?}", existing_byte, next_byte);
}

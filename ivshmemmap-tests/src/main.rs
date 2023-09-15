extern crate ivshmemmap;

fn main() {
    println!("Testing driver...");
    let mut device = ivshmemmap::windows::fetch_ivshmem_devices(|mut dev| {
        dev.remove(1)
    }).unwrap();

    let existing_byte = device.direct()[1];
    println!("BEFORE: {:?}", existing_byte);
    device.set_all_bytes(existing_byte + 1).unwrap();
    println!("AFTER: {:?}", device.direct()[1]);
}

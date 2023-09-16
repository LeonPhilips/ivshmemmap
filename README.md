# ivshmemmap
Library for usage with inter-vm shared memory.
See also: https://github.com/virtio-win/kvm-guest-drivers-windows/tree/master/ivshmem

This library provides a safe wrapper for using the shared memory as if it's a `&'static [u8]` object.

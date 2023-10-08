# ivshmemmap
Library for usage with inter-vm shared memory.

This library provides a wrapper for using the shared memory as if it's a `&'static [u8]` object.

See also: [virtio-win/kvm-guest-drivers-windows/ivshmem/](https://github.com/virtio-win/kvm-guest-drivers-windows/tree/master/ivshmem)

# Example usage
[ivshmemmap/ivshmemmap-tests/src/main.rs](https://github.com/TerminatorNL/ivshmemmap/tree/master/ivshmemmap-tests/src/main.rs)

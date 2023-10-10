pub mod allocator;
pub mod allocator_types;

mod utils;

#[derive(Clone, Copy)]
pub struct AllocationSizes {
    device_memory_block_size: u64,
    host_memory_block_size: u64,
}

impl Default for AllocationSizes {
    fn default() -> Self {
        Self {
            device_memory_block_size: 256 * 1024 * 1024,
            host_memory_block_size: 64 * 1024 * 1024,
        }
    }
}

use ash::vk;

pub mod dedicated;
pub mod free_list;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum MemoryLocation {
    #[default]
    Unknown,
    // Let driver decide.
    Gpu,
    // GPU only.
    CpuToGpu,
    // Used for transferring data to gpu.
    GpuToCpu, // Used for transferring data from gpu.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AllocationType {
    Free,
    Linear,
    NonLinear,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum AllocationScheme {
    // Driver managed.
    DedicatedBuffer(vk::Buffer),
    DedicatedImage(vk::Image),

    // Allocator managed.
    #[default]
    GpuAllocatorManaged,
}

pub trait SubAllocator: Sync + Send {
    fn allocate(
        &mut self,
        size: u64,
        alignment: u64,
        allocation_type: AllocationType,
        granularity: u64,
        name: &str,
    ) -> (u64, std::num::NonZeroU64);

    fn free(&mut self, chunk_id: Option<std::num::NonZeroU64>);

    fn report_memory_leaks(&self, memory_type_index: usize, memory_block_index: usize);

    #[must_use]
    fn supports_general_allocations(&self) -> bool;
    #[must_use]
    fn size(&self) -> u64;
    #[must_use]
    fn allocated(&self) -> u64;

    #[must_use]
    fn available_memory(&self) -> u64 {
        self.size() - self.allocated()
    }

    #[must_use]
    fn is_empty(&self) -> bool {
        self.allocated() == 0
    }
}

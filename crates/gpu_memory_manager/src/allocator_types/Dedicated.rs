use crate::allocator_types::{AllocationType, SubAllocator};

pub struct DedicatedBlockAllocator {
    size: u64,
    allocated: u64,
    name: Option<String>,
}

impl DedicatedBlockAllocator {
    pub fn new(size: u64) -> Self {
        Self {
            size,
            allocated: 0,
            name: None,
        }
    }
}

impl SubAllocator for DedicatedBlockAllocator {
    fn allocate(
        &mut self,
        size: u64,
        _alignment: u64,
        _allocation_type: AllocationType,
        _granularity: u64,
        name: &str,
    ) -> (u64, std::num::NonZeroU64) {
        if self.allocated != 0 {
            panic!("DedicatedBlockAllocator: out of memory.");
        }
        if self.size != size {
            panic!("DedicatedBlockAllocator: size must be equal to the size of the allocator.");
        }

        self.allocated = size;
        self.name = Some(name.to_string());

        let dummy_id = std::num::NonZeroU64::new(1).unwrap();
        (0, dummy_id)
    }

    fn free(&mut self, chunk_id: Option<std::num::NonZeroU64>) {
        if chunk_id != std::num::NonZeroU64::new(1) {
            panic!("Chunk ID must be 1.")
        } else {
            self.allocated = 0;
        }
    }

    fn report_memory_leaks(&self, memory_type_index: usize, memory_block_index: usize) {
        let empty = "".to_string();
        let name = self.name.as_ref().unwrap_or(&empty);

        println!("Leak detected: type {memory_type_index}, block {memory_block_index}, dedicated allocation: size: 0x{}, name: {name}", self.size);
        // TODO: Logging...
    }

    fn supports_general_allocations(&self) -> bool {
        false
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn allocated(&self) -> u64 {
        self.allocated
    }
}

use ash::vk;

use crate::allocator_types::dedicated::DedicatedBlockAllocator;
use crate::allocator_types::free_list::FreeListAllocator;
use crate::allocator_types::{AllocationScheme, AllocationType, MemoryLocation};
use crate::{allocator_types, AllocationSizes};

pub struct AllocatorCreateInfo {
    instance: ash::Instance,
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    buffer_device_address: bool,
    allocation_sizes: AllocationSizes,
}

impl AllocatorCreateInfo {
    pub fn builder() -> AllocatorCreateInfoBuilder {
        AllocatorCreateInfoBuilder::new()
    }
}

#[derive(Default)]
pub struct AllocatorCreateInfoBuilder {
    instance: Option<ash::Instance>,
    device: Option<ash::Device>,
    physical_device: Option<vk::PhysicalDevice>,
    buffer_device_address: bool,
    allocation_sizes: AllocationSizes,
}

impl AllocatorCreateInfoBuilder {
    pub fn new() -> Self {
        Self {
            instance: None,
            device: None,
            physical_device: None,
            buffer_device_address: false,
            allocation_sizes: AllocationSizes::default(),
        }
    }
    pub fn instance(mut self, instance: ash::Instance) -> Self {
        self.instance = Some(instance);
        self
    }
    pub fn device(mut self, device: ash::Device) -> Self {
        self.device = Some(device);
        self
    }
    pub fn physical_device(mut self, physical_device: vk::PhysicalDevice) -> Self {
        self.physical_device = Some(physical_device);
        self
    }
    pub fn buffer_device_address(mut self, buffer_device_address: bool) -> Self {
        self.buffer_device_address = buffer_device_address;
        self
    }
    pub fn allocation_sizes(mut self, allocation_sizes: AllocationSizes) -> Self {
        self.allocation_sizes = allocation_sizes;
        self
    }
    pub fn build(self) -> AllocatorCreateInfo {
        if self.instance.is_none() {
            panic!("AllocatorCreateInfoBuilder: instance is not set."); // TODO: maybe dont panic?
        }
        if self.device.is_none() {
            panic!("AllocatorCreateInfoBuilder: device is not set.");
        }
        if self.physical_device.is_none() {
            panic!("AllocatorCreateInfoBuilder: physical_device is not set.");
        }

        AllocatorCreateInfo {
            instance: self.instance.unwrap(),
            device: self.device.unwrap(),
            physical_device: self.physical_device.unwrap(),
            buffer_device_address: self.buffer_device_address,
            allocation_sizes: self.allocation_sizes,
        }
    }
}

pub struct Allocator {
    memory_types: Vec<MemoryType>,
    memory_heaps: Vec<vk::MemoryHeap>,
    device: ash::Device,
    buffer_image_granularity: u64,
    allocation_sizes: AllocationSizes,
}

impl Allocator {
    pub fn new(info: &AllocatorCreateInfo) -> Self {
        let mem_props = unsafe {
            info.instance
                .get_physical_device_memory_properties(info.physical_device)
        };

        let memory_types = &mem_props.memory_types[..mem_props.memory_type_count as _];
        let memory_heaps = mem_props.memory_heaps[..mem_props.memory_heap_count as _].to_vec();

        let memory_types = memory_types
            .iter()
            .enumerate()
            .map(|(i, mem_type)| MemoryType {
                memory_blocks: Vec::default(),
                memory_properties: mem_type.property_flags,
                memory_type_index: i,
                heap_index: mem_type.heap_index as usize,
                mappable: mem_type
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::HOST_VISIBLE),
                active_general_blocks: 0,
                buffer_device_address: info.buffer_device_address,
            })
            .collect::<Vec<_>>();

        let physical_device_properties = unsafe {
            info.instance
                .get_physical_device_properties(info.physical_device)
        };

        let granularity = physical_device_properties.limits.buffer_image_granularity;

        Self {
            memory_types,
            memory_heaps,
            device: info.device.clone(),
            buffer_image_granularity: granularity,
            allocation_sizes: info.allocation_sizes,
        }
    }

    pub fn allocate(&mut self, info: &AllocationCreateInfo<'_>) -> Allocation {
        let size = info.requirements.size;
        let alignment = info.requirements.alignment;

        if size == 0 || !alignment.is_power_of_two() {
            panic!(
                "Invalid allocation size or alignment. Must be greater than 0 and a power of two."
            );
        }

        let mem_loc_preferred_bits = match info.location {
            MemoryLocation::Gpu => vk::MemoryPropertyFlags::DEVICE_LOCAL,
            MemoryLocation::CpuToGpu => {
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT
                    | vk::MemoryPropertyFlags::DEVICE_LOCAL
            }
            MemoryLocation::GpuToCpu => {
                vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::HOST_COHERENT
                    | vk::MemoryPropertyFlags::HOST_CACHED
            }
            MemoryLocation::Unknown => vk::MemoryPropertyFlags::empty(),
        };
        let mut memory_type_index_opt =
            self.find_memory_type_index(&info.requirements, mem_loc_preferred_bits);

        if memory_type_index_opt.is_none() {
            let mem_loc_required_bits = match info.location {
                MemoryLocation::Gpu => vk::MemoryPropertyFlags::DEVICE_LOCAL,
                MemoryLocation::CpuToGpu | MemoryLocation::GpuToCpu => {
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
                }
                MemoryLocation::Unknown => vk::MemoryPropertyFlags::empty(),
            };

            memory_type_index_opt =
                self.find_memory_type_index(&info.requirements, mem_loc_required_bits);
        }

        let memory_type_index =
            memory_type_index_opt.expect("No compatible memory type found") as usize;

        // Do not try to create a block if the heap is smaller than the required size (avoids validation warnings).
        let memory_type = &mut self.memory_types[memory_type_index];
        let allocation = if size > self.memory_heaps[memory_type.heap_index].size {
            None
        } else {
            Some(memory_type.allocate(
                &self.device,
                info,
                self.buffer_image_granularity,
                &self.allocation_sizes,
            ))
        };

        if info.location == MemoryLocation::CpuToGpu {
            if let Some(allocation) = allocation {
                allocation
            } else {
                let mem_loc_preferred_bits =
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

                let memory_type_index_opt =
                    self.find_memory_type_index(&info.requirements, mem_loc_preferred_bits);

                let memory_type_index =
                    memory_type_index_opt.expect("No compatible memory type found") as usize;

                self.memory_types[memory_type_index].allocate(
                    &self.device,
                    info,
                    self.buffer_image_granularity,
                    &self.allocation_sizes,
                )
            }
        } else {
            allocation.unwrap()
        }
    }

    pub fn free(&mut self, allocation: Allocation) {
        if allocation.is_null() {
            return;
        }

        self.memory_types[allocation.memory_type_index].free(allocation, &self.device);
    }

    fn find_memory_type_index(
        &self,
        memory_req: &vk::MemoryRequirements,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        self.memory_types
            .iter()
            .find(|memory_type| {
                (1 << memory_type.memory_type_index) & memory_req.memory_type_bits != 0
                    && memory_type.memory_properties.contains(flags)
            })
            .map(|memory_type| memory_type.memory_type_index as _)
    }

    pub fn report_memory_leaks(&self) {
        for (mem_type_i, mem_type) in self.memory_types.iter().enumerate() {
            for (block_i, mem_block) in mem_type.memory_blocks.iter().enumerate() {
                if let Some(mem_block) = mem_block {
                    mem_block
                        .sub_allocator
                        .report_memory_leaks(mem_type_i, block_i);
                }
            }
        }
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        self.report_memory_leaks();

        // Free all remaining memory blocks
        for mem_type in self.memory_types.iter_mut() {
            for mem_block in mem_type.memory_blocks.iter_mut() {
                let block = mem_block.take();
                if let Some(block) = block {
                    block.destroy(&self.device);
                }
            }
        }
    }
}

pub struct MemoryType {
    memory_blocks: Vec<Option<MemoryBlock>>,
    memory_properties: vk::MemoryPropertyFlags,
    memory_type_index: usize,
    heap_index: usize,
    mappable: bool,
    active_general_blocks: usize,
    buffer_device_address: bool,
}

impl MemoryType {
    fn allocate(
        &mut self,
        device: &ash::Device,
        info: &AllocationCreateInfo<'_>,
        granularity: u64,
        allocation_sizes: &AllocationSizes,
    ) -> Allocation {
        let allocation_type = if info.linear {
            AllocationType::Linear
        } else {
            AllocationType::NonLinear
        };

        let memory_block_size = if self
            .memory_properties
            .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
        {
            allocation_sizes.host_memory_block_size
        } else {
            allocation_sizes.device_memory_block_size
        };

        let size = info.requirements.size;
        let alignment = info.requirements.alignment;

        let dedicated_allocation = info.allocation_scheme != AllocationScheme::GpuAllocatorManaged;
        let requires_personal_block = size > memory_block_size;

        // Create a dedicated block for large memory allocations or allocations that require dedicated memory allocations.
        if dedicated_allocation || requires_personal_block {
            let memory_block = MemoryBlock::new(
                device,
                size,
                self.memory_type_index,
                self.mappable,
                self.buffer_device_address,
                info.allocation_scheme,
                requires_personal_block,
            );

            let mut block_index = None;
            for (i, block) in self.memory_blocks.iter().enumerate() {
                if block.is_none() {
                    block_index = Some(i);
                    break;
                }
            }

            let block_index = match block_index {
                Some(i) => {
                    self.memory_blocks[i].replace(memory_block);
                    i
                }
                None => {
                    self.memory_blocks.push(Some(memory_block));
                    self.memory_blocks.len() - 1
                }
            };

            let memory_block = self.memory_blocks[block_index]
                .as_mut()
                .expect("Memory block must be Some");

            let (offset, chunk_id) = memory_block.sub_allocator.allocate(
                size,
                alignment,
                allocation_type,
                granularity,
                info.name,
            );

            return Allocation {
                chunk_id: Some(chunk_id),
                offset,
                size,
                memory_block_index: block_index,
                memory_type_index: self.memory_type_index,
                device_memory: memory_block.device_memory,
                mapped_ptr: memory_block.mapped_ptr,
                memory_properties: self.memory_properties,
                dedicated_allocation,
            };
        }

        let mut empty_block_index = None;
        for (memory_block_idx, memory_block) in self.memory_blocks.iter_mut().enumerate().rev() {
            if let Some(memory_block) = memory_block {
                let (offset, chunk_id) = memory_block.sub_allocator.allocate(
                    size,
                    alignment,
                    allocation_type,
                    granularity,
                    info.name,
                );

                let mapped_ptr = get_mapped_ptr(memory_block, offset as usize);
                return Allocation {
                    chunk_id: Some(chunk_id),
                    offset,
                    size,
                    memory_block_index: memory_block_idx,
                    memory_type_index: self.memory_type_index,
                    device_memory: memory_block.device_memory,
                    memory_properties: self.memory_properties,
                    mapped_ptr,
                    dedicated_allocation: false,
                };
            } else if empty_block_index.is_none() {
                empty_block_index = Some(memory_block_idx);
            }
        }

        let new_memory_block = MemoryBlock::new(
            device,
            memory_block_size,
            self.memory_type_index,
            self.mappable,
            self.buffer_device_address,
            info.allocation_scheme,
            false,
        );

        let new_block_index = if let Some(block_index) = empty_block_index {
            self.memory_blocks[block_index] = Some(new_memory_block);
            block_index
        } else {
            self.memory_blocks.push(Some(new_memory_block));
            self.memory_blocks.len() - 1
        };

        self.active_general_blocks += 1;

        let memory_block = self.memory_blocks[new_block_index]
            .as_mut()
            .expect("Memory block must be Some");
        let (offset, chunk_id) = memory_block.sub_allocator.allocate(
            size,
            alignment,
            allocation_type,
            granularity,
            info.name,
        );

        let mapped_ptr = get_mapped_ptr(memory_block, offset as usize);

        Allocation {
            chunk_id: Some(chunk_id),
            offset,
            size,
            memory_block_index: new_block_index,
            memory_type_index: self.memory_type_index,
            device_memory: memory_block.device_memory,
            mapped_ptr,
            memory_properties: self.memory_properties,
            dedicated_allocation: false,
        }
    }

    fn free(&mut self, allocation: Allocation, device: &ash::Device) {
        let block_idx = allocation.memory_block_index;

        let mem_block = self.memory_blocks[block_idx]
            .as_mut()
            .expect("Memory block must be Some");

        mem_block.sub_allocator.free(allocation.chunk_id);

        if mem_block.sub_allocator.is_empty() {
            if mem_block.sub_allocator.supports_general_allocations() {
                if self.active_general_blocks > 1 {
                    let block = self.memory_blocks[block_idx].take();
                    let block = block.expect("Memory block must be Some");
                    block.destroy(device);

                    self.active_general_blocks -= 1;
                }
            } else {
                let block = self.memory_blocks[block_idx].take();
                let block = block.expect("Memory block must be Some");
                block.destroy(device);
            }
        }
    }
}

pub struct MemoryBlock {
    device_memory: vk::DeviceMemory,
    mapped_ptr: Option<SendSyncPtr>,
    sub_allocator: Box<dyn allocator_types::SubAllocator>,
}

impl MemoryBlock {
    fn new(
        device: &ash::Device,
        size: u64,
        mem_type_index: usize,
        mapped: bool,
        buffer_device_address: bool,
        allocation_scheme: AllocationScheme,
        requires_personal_block: bool,
    ) -> Self {
        let device_memory = {
            let alloc_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(size)
                .memory_type_index(mem_type_index as u32);

            let allocation_flags = vk::MemoryAllocateFlags::DEVICE_ADDRESS;
            let mut flags_info = vk::MemoryAllocateFlagsInfo::builder().flags(allocation_flags);
            let alloc_info = if buffer_device_address {
                alloc_info.push_next(&mut flags_info)
            } else {
                alloc_info
            };

            // Flag the memory as dedicated if required.
            let mut dedicated_memory_info = vk::MemoryDedicatedAllocateInfo::builder();
            let alloc_info = match allocation_scheme {
                AllocationScheme::DedicatedBuffer(buffer) => {
                    dedicated_memory_info = dedicated_memory_info.buffer(buffer);
                    alloc_info.push_next(&mut dedicated_memory_info)
                }
                AllocationScheme::DedicatedImage(image) => {
                    dedicated_memory_info = dedicated_memory_info.image(image);
                    alloc_info.push_next(&mut dedicated_memory_info)
                }
                AllocationScheme::GpuAllocatorManaged => alloc_info,
            };

            unsafe { device.allocate_memory(&alloc_info, None) }
                .expect("Device memory allocation failed")
        };

        let mapped_ptr = mapped.then(|| {
            std::ptr::NonNull::new(
                unsafe {
                    device.map_memory(
                        device_memory,
                        0,
                        vk::WHOLE_SIZE,
                        vk::MemoryMapFlags::empty(),
                    )
                }
                .map_err(|e| {
                    unsafe { device.free_memory(device_memory, None) };
                    e
                })
                .expect("Failed to map memory"),
            )
            .map(SendSyncPtr)
            .expect("Returned memory was null")
        });

        let sub_allocator: Box<dyn allocator_types::SubAllocator> = if allocation_scheme
            != AllocationScheme::GpuAllocatorManaged
            || requires_personal_block
        {
            Box::new(DedicatedBlockAllocator::new(size))
        } else {
            Box::new(FreeListAllocator::new(size))
        };

        Self {
            device_memory,
            mapped_ptr,
            sub_allocator,
        }
    }

    fn destroy(self, device: &ash::Device) {
        if self.mapped_ptr.is_some() {
            unsafe { device.unmap_memory(self.device_memory) };
        }

        unsafe { device.free_memory(self.device_memory, None) };
    }
}

#[derive(Default)]
pub struct AllocationCreateInfo<'a> {
    pub name: &'a str,
    pub requirements: vk::MemoryRequirements,
    pub location: MemoryLocation,
    pub linear: bool,
    pub allocation_scheme: AllocationScheme,
}

impl<'a> AllocationCreateInfo<'a> {
    pub fn builder() -> AllocationCreateDescBuilder<'a> {
        AllocationCreateDescBuilder::new()
    }
}

#[derive(Default)]
pub struct AllocationCreateDescBuilder<'a> {
    inner: AllocationCreateInfo<'a>,
}

impl<'a> AllocationCreateDescBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn name(mut self, name: &'a str) -> Self {
        self.inner.name = name;
        self
    }
    pub fn requirements(mut self, requirements: vk::MemoryRequirements) -> Self {
        self.inner.requirements = requirements;
        self
    }
    pub fn location(mut self, location: MemoryLocation) -> Self {
        self.inner.location = location;
        self
    }
    pub fn linear(mut self, linear: bool) -> Self {
        self.inner.linear = linear;
        self
    }
    pub fn allocation_scheme(mut self, allocation_scheme: AllocationScheme) -> Self {
        self.inner.allocation_scheme = allocation_scheme;
        self
    }
    pub fn build(self) -> AllocationCreateInfo<'a> {
        self.inner
    }
}

pub struct Allocation {
    chunk_id: Option<std::num::NonZeroU64>,
    offset: u64,
    size: u64,
    memory_block_index: usize,
    memory_type_index: usize,
    device_memory: vk::DeviceMemory,
    mapped_ptr: Option<SendSyncPtr>,
    dedicated_allocation: bool,
    memory_properties: vk::MemoryPropertyFlags,
}

impl Allocation {
    pub fn chunk_id(&self) -> Option<std::num::NonZeroU64> {
        self.chunk_id
    }
    pub fn memory_properties(&self) -> vk::MemoryPropertyFlags {
        self.memory_properties
    }
    pub fn is_dedicated(&self) -> bool {
        self.dedicated_allocation
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn is_null(&self) -> bool {
        self.chunk_id.is_none()
    }
    pub fn mapped_ptr(&self) -> Option<std::ptr::NonNull<std::ffi::c_void>> {
        self.mapped_ptr.map(|SendSyncPtr(p)| p)
    }
    pub fn mapped_slice(&self) -> Option<&[u8]> {
        self.mapped_ptr().map(|ptr| unsafe {
            std::slice::from_raw_parts(ptr.cast().as_ptr(), self.size as usize)
        })
    }
    pub fn mapped_slice_mut(&mut self) -> Option<&mut [u8]> {
        self.mapped_ptr().map(|ptr| unsafe {
            std::slice::from_raw_parts_mut(ptr.cast().as_ptr(), self.size as usize)
        })
    }

    /// Returns the memory block of the allocation.
    /// Exposed for the use in [`ash::Device::bind_buffer_memory()`]
    ///
    /// # Safety
    /// Do not allocate or free the memory block.
    pub unsafe fn memory(&self) -> vk::DeviceMemory {
        self.device_memory
    }
}

#[derive(Clone, Copy)]
struct SendSyncPtr(std::ptr::NonNull<std::ffi::c_void>);

unsafe impl Send for SendSyncPtr {}

unsafe impl Sync for SendSyncPtr {}

fn get_mapped_ptr(memory_block: &MemoryBlock, offset: usize) -> Option<SendSyncPtr> {
    if let Some(SendSyncPtr(mapped_ptr)) = memory_block.mapped_ptr {
        let offset_ptr = unsafe { mapped_ptr.as_ptr().add(offset) };
        std::ptr::NonNull::new(offset_ptr).map(SendSyncPtr)
    } else {
        None
    }
}

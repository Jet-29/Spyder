use std::collections::{HashMap, HashSet};

use crate::allocator_types::{AllocationType, SubAllocator};
use crate::utils::{align_up, has_granularity_conflict, is_on_same_page};

struct MemoryChunk {
    chunk_id: std::num::NonZeroU64,
    size: u64,
    offset: u64,
    allocation_type: AllocationType,
    name: Option<String>,
    next: Option<std::num::NonZeroU64>,
    prev: Option<std::num::NonZeroU64>,
}

pub struct FreeListAllocator {
    size: u64,
    allocated: u64,
    chunk_id_counter: u64,
    chunks: HashMap<std::num::NonZeroU64, MemoryChunk>,
    free_chunks: HashSet<std::num::NonZeroU64>,
}

impl FreeListAllocator {
    pub fn new(size: u64) -> Self {
        let initial_chunk_id = std::num::NonZeroU64::new(1).unwrap();

        let mut chunks = HashMap::default();

        chunks.insert(
            initial_chunk_id,
            MemoryChunk {
                chunk_id: initial_chunk_id,
                size,
                offset: 0,
                allocation_type: AllocationType::Free,
                name: None,
                prev: None,
                next: None,
            },
        );

        let mut free_chunks = HashSet::default();
        free_chunks.insert(initial_chunk_id);

        Self {
            size,
            allocated: 0,
            chunk_id_counter: 2,
            chunks,
            free_chunks,
        }
    }

    fn get_new_chunk_id(&mut self) -> std::num::NonZeroU64 {
        if self.chunk_id_counter == u64::MAX {
            panic!("Out of memory.")
        }

        let id = self.chunk_id_counter;
        self.chunk_id_counter += 1;
        std::num::NonZeroU64::new(id).expect("New chunk id was 0")
    }
    fn remove_id_from_free_list(&mut self, chunk_id: std::num::NonZeroU64) {
        self.free_chunks.remove(&chunk_id);
    }
    fn merge_free_chunks(
        &mut self,
        chunk_left: std::num::NonZeroU64,
        chunk_right: std::num::NonZeroU64,
    ) {
        // Gather data from right chunk and remove it
        let (right_size, right_next) = {
            let chunk = self
                .chunks
                .remove(&chunk_right)
                .expect("Chunk ID not present in chunk list");
            self.remove_id_from_free_list(chunk.chunk_id);

            (chunk.size, chunk.next)
        };

        // Merge into left chunk
        {
            let chunk = self
                .chunks
                .get_mut(&chunk_left)
                .expect("Chunk ID not present in chunk list");
            chunk.next = right_next;
            chunk.size += right_size;
        }

        // Patch pointers
        if let Some(right_next) = right_next {
            let chunk = self
                .chunks
                .get_mut(&right_next)
                .expect("Chunk ID not present in chunk list");
            chunk.prev = Some(chunk_left);
        }
    }
}

impl SubAllocator for FreeListAllocator {
    fn allocate(
        &mut self,
        size: u64,
        alignment: u64,
        allocation_type: AllocationType,
        granularity: u64,
        name: &str,
    ) -> (u64, std::num::NonZeroU64) {
        let free_size = self.size - self.allocated;
        if size > free_size {
            panic!("Out of memory.")
        }

        let mut best_fit_id: Option<std::num::NonZeroU64> = None;
        let mut best_offset = 0u64;
        let mut best_aligned_size = 0u64;
        let mut best_chunk_size = 0u64;

        for current_chunk_id in self.free_chunks.iter() {
            let current_chunk = self
                .chunks
                .get(current_chunk_id)
                .expect("Chunk ID not present in chunk list");

            if current_chunk.size < size {
                continue;
            }

            let mut offset = align_up(current_chunk.offset, alignment);

            if let Some(prev_idx) = current_chunk.prev {
                let previous = self
                    .chunks
                    .get(&prev_idx)
                    .expect("Invalid chunk reference.");
                if is_on_same_page(previous.offset, previous.size, offset, granularity)
                    && has_granularity_conflict(
                        previous.allocation_type.clone(),
                        allocation_type.clone(),
                    )
                {
                    offset = align_up(offset, granularity);
                }
            }

            let padding = offset - current_chunk.offset;
            let aligned_size = padding + size;

            if aligned_size > current_chunk.size {
                continue;
            }

            if let Some(next_idx) = current_chunk.next {
                let next = self
                    .chunks
                    .get(&next_idx)
                    .expect("Invalid next chunk reference.");
                if is_on_same_page(offset, size, next.offset, granularity)
                    && has_granularity_conflict(
                        allocation_type.clone(),
                        next.allocation_type.clone(),
                    )
                {
                    continue;
                }
            }

            if best_fit_id.is_none() || current_chunk.size < best_chunk_size {
                best_fit_id = Some(*current_chunk_id);
                best_aligned_size = aligned_size;
                best_offset = offset;

                best_chunk_size = current_chunk.size;
            };
        }

        let first_fit_id = best_fit_id.expect("Out of memory");

        let chunk_id = if best_chunk_size > best_aligned_size {
            let new_chunk_id = self.get_new_chunk_id();

            let new_chunk = {
                let free_chunk = self
                    .chunks
                    .get_mut(&first_fit_id)
                    .expect("Chunk id but be in chunk list");
                let new_chunk = MemoryChunk {
                    chunk_id: new_chunk_id,
                    size: best_aligned_size,
                    offset: free_chunk.offset,
                    allocation_type,
                    name: Some(name.to_string()),
                    prev: free_chunk.prev,
                    next: Some(first_fit_id),
                };

                free_chunk.prev = Some(new_chunk.chunk_id);
                free_chunk.offset += best_aligned_size;
                free_chunk.size -= best_aligned_size;
                new_chunk
            };

            if let Some(prev_id) = new_chunk.prev {
                let prev_chunk = self
                    .chunks
                    .get_mut(&prev_id)
                    .expect("Invalid previous chunk reference.");
                prev_chunk.next = Some(new_chunk.chunk_id);
            }

            self.chunks.insert(new_chunk_id, new_chunk);

            new_chunk_id
        } else {
            let chunk = self
                .chunks
                .get_mut(&first_fit_id)
                .expect("Invalid chunk reference.");

            chunk.allocation_type = allocation_type;
            chunk.name = Some(name.to_string());

            self.remove_id_from_free_list(first_fit_id);

            first_fit_id
        };

        self.allocated += best_aligned_size;

        (best_offset, chunk_id)
    }

    fn free(&mut self, chunk_id: Option<std::num::NonZeroU64>) {
        let chunk_id = chunk_id.expect("Chunk id must be a valid reference");

        let (next_id, prev_id) = {
            let chunk = self
                .chunks
                .get_mut(&chunk_id)
                .expect("Attempting to free a chunk that is not in the chunk list");
            chunk.allocation_type = AllocationType::Free;
            chunk.name = None;
            self.allocated -= chunk.size;
            self.free_chunks.insert(chunk.chunk_id);
            (chunk.next, chunk.prev)
        };

        if let Some(next_id) = next_id {
            if self.chunks[&next_id].allocation_type == AllocationType::Free {
                self.merge_free_chunks(chunk_id, next_id);
            }
        }

        if let Some(prev_id) = prev_id {
            if self.chunks[&prev_id].allocation_type == AllocationType::Free {
                self.merge_free_chunks(prev_id, chunk_id);
            }
        }
    }

    fn report_memory_leaks(&self, memory_type_index: usize, memory_block_index: usize) {
        for (chunk_id, chunk) in self.chunks.iter() {
            if chunk.allocation_type == AllocationType::Free {
                continue;
            }
            let empty = "".to_string();
            let name = chunk.name.as_ref().unwrap_or(&empty);

            println!("Leak detected: type {memory_type_index}, block {memory_block_index}, chunk: {chunk_id}, size: {}, offset {}, allocation type {:?}, name: {name}", chunk.size, chunk.offset, chunk.allocation_type);
            // TODO: Logging...
        }
    }

    fn supports_general_allocations(&self) -> bool {
        true
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn allocated(&self) -> u64 {
        self.allocated
    }
}

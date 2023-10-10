use crate::allocator_types::AllocationType;

pub fn align_down(val: u64, alignment: u64) -> u64 {
    val & !(alignment - 1u64)
}

pub fn align_up(val: u64, alignment: u64) -> u64 {
    align_down(val + alignment - 1u64, alignment)
}

pub fn is_on_same_page(offset_a: u64, size_a: u64, offset_b: u64, page_size: u64) -> bool {
    let end_a = offset_a + size_a - 1;
    let end_page_a = align_down(end_a, page_size);
    let start_b = offset_b;
    let start_page_b = align_down(start_b, page_size);

    end_page_a == start_page_b
}

pub fn has_granularity_conflict(type0: AllocationType, type1: AllocationType) -> bool {
    if type0 == AllocationType::Free || type1 == AllocationType::Free {
        return false;
    }

    type0 != type1
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use util_macros::bitflags;

    // Use 9 flags to push it over the u8 limit
    #[bitflags]
    enum TestFlags {
        F1,
        F2,
        F3,
        F4,
        F5,
        F6,
        F7,
        F8,
        F9,
    }

    #[test]
    fn test_bitflags() {
        let mut flags = TestFlags::F1 | TestFlags::F2;
        assert_eq!(flags.bits(), 3);

        flags |= TestFlags::F3;
        assert_eq!(flags.bits(), 7);

        flags -= TestFlags::F2;
        assert_eq!(flags.bits(), 5);

        flags ^= TestFlags::F1;
        assert_eq!(flags.bits(), 4);

        assert_eq!(flags.bits().type_id(), std::any::TypeId::of::<u16>())
    }
}

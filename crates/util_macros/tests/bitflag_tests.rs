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

    #[test]
    fn test_from_bits() {
        let flags = TestFlags::from_bits(0b0101);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F3);

        // Should remove on invalid flags
        let flags = TestFlags::from_bits(0b1000000101);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F3);
    }

    #[test]
    fn test_empty_full() {
        assert!(TestFlags::EMPTY.is_empty());
        assert!(TestFlags::FULL.is_full());
        assert_eq!(TestFlags::FULL, !TestFlags::EMPTY);
        assert_eq!(!TestFlags::FULL, TestFlags::EMPTY);
        assert_eq!(!!TestFlags::FULL, TestFlags::FULL);
        assert_eq!(!!TestFlags::EMPTY, TestFlags::EMPTY);
    }

    #[test]
    fn test_set() {
        let mut flags = TestFlags::F1 | TestFlags::F2;
        flags.set(TestFlags::F3, true);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F2 | TestFlags::F3);

        flags.set(TestFlags::F2, false);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F3);

        flags.set(TestFlags::F1, false);
        assert_eq!(flags, TestFlags::F3);

        flags.set(TestFlags::F3, false);
        assert_eq!(flags, TestFlags::EMPTY);
    }

    #[test]
    fn intersection() {
        let mut flags = TestFlags::F1 | TestFlags::F2 | TestFlags::F3;

        assert_eq!(flags.intersection(TestFlags::F1), TestFlags::F1);
        assert!(flags.intersects(TestFlags::F3));

        assert!(flags.contains_all(TestFlags::F1 | TestFlags::F2));
        assert!(!flags.contains_all(TestFlags::F1 | TestFlags::F2 | TestFlags::F3 | TestFlags::F4));
        assert!(flags.contains_all(TestFlags::F1 | TestFlags::F2 | TestFlags::F3));

        flags.keep_intersection(TestFlags::F2 | TestFlags::F3 | TestFlags::F4);
        assert_eq!(flags, TestFlags::F2 | TestFlags::F3);

        flags.keep_intersection(TestFlags::F1);
        assert_eq!(flags, TestFlags::EMPTY);
    }

    #[test]
    fn test_union() {
        let mut flags = TestFlags::F1 | TestFlags::F2;
        flags.insert(TestFlags::F3);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F2 | TestFlags::F3);

        flags.insert(TestFlags::F2);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F2 | TestFlags::F3);

        flags.insert(TestFlags::F4);
        assert_eq!(
            flags,
            TestFlags::F1 | TestFlags::F2 | TestFlags::F3 | TestFlags::F4
        );

        flags.remove(TestFlags::F2);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F3 | TestFlags::F4);
    }

    #[test]
    fn test_toggle() {
        let mut flags = TestFlags::F1 | TestFlags::F2;
        flags.toggle(TestFlags::F3);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F2 | TestFlags::F3);

        flags.toggle(TestFlags::F2);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F3);

        flags.toggle(TestFlags::F4);
        assert_eq!(flags, TestFlags::F1 | TestFlags::F3 | TestFlags::F4);

        flags.toggle(TestFlags::F2);
        assert_eq!(
            flags,
            TestFlags::F1 | TestFlags::F2 | TestFlags::F3 | TestFlags::F4
        );
    }
}

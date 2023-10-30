#[cfg(test)]
mod tests {
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
    fn bitor() {
        assert_eq!((TestFlags::F1 | TestFlags::F2).get_value(), 3)
    }

    #[test]
    fn bitor_assign() {
        let mut my_flags = TestFlags::F3;
        my_flags |= TestFlags::F4;
        assert_eq!(my_flags, TestFlags::F3 | TestFlags::F4);
    }

    #[test]
    fn bitand() {
        assert!(!((TestFlags::F2 | TestFlags::F1) & TestFlags::F3));
        assert!((TestFlags::F7 | TestFlags::F9 | TestFlags::F8) & TestFlags::F7);
    }
}

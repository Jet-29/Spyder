#[cfg(test)]
mod test {
    use resource_manager::prelude::ResourceManager;

    #[derive(Debug, PartialEq, Eq)]
    struct Foo;

    #[derive(Debug, PartialEq, Eq)]
    struct Bar {
        data: u32,
    }

    fn get_resource_manager() -> ResourceManager {
        let mut rs = ResourceManager::new();
        rs.add(Foo);
        rs.add(Bar { data: 42 });
        rs
    }

    #[test]
    fn getting_data_back() {
        let mut rs = get_resource_manager();

        let foo_owned = rs.remove::<Foo>();
        let bar_data = rs.get::<Bar>().unwrap().data;
        assert_eq!(foo_owned, Some(Foo));
        assert_eq!(bar_data, 42);
    }

    #[test]
    #[should_panic]
    fn unchecked_methods() {
        let rs = ResourceManager::new();
        rs.get_unchecked::<Foo>();
    }
}

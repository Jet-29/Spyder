use resource_manager::prelude::ResourceManager;

struct Foo {
    data: u32,
}

struct Bar {
    data: u32,
}

fn main() {
    let mut rs = ResourceManager::new();
    rs.add(Foo { data: 42 });
}

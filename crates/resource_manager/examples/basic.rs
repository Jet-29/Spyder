use resource_manager::prelude::ResourceManager;

#[derive(Debug)]
struct Foo;

#[derive(Debug)]
struct Bar {
    data: u32,
}

fn main() {
    let mut rs = ResourceManager::new();
    rs.add(Foo);
    rs.add(Bar { data: 42 });

    let foo_owned = rs.remove::<Foo>();
    let bar_data = rs.get::<Bar>().unwrap().data;
    println!("{foo_owned:?}");
    println!("bar_data = {bar_data}");
}

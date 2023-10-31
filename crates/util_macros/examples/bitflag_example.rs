use util_macros::bitflags;

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

fn main() {
    let flag = TestFlags::F1 | TestFlags::F2;

    match flag {
        _ if flag == TestFlags::F1 => println!("F1"),
        _ if flag == TestFlags::F1 | TestFlags::F2 => println!("F1 | F2"),
        TestFlags::F2 => println!("F2"),
        TestFlags::F3 => println!("F3"),
        f => {
            dbg!(f);
        }
    }

    println!("{}", TestFlags::FULL)
}

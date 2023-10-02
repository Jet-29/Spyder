use renderer_macros::include_glsl;

fn main() {
    let input = include_glsl!("examples/test.vert");
    println!("{:?}", input)
}

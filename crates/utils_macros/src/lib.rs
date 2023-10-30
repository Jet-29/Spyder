use proc_macro::TokenStream;

mod bitflags;

#[proc_macro_attribute]
pub fn bitflags(_: TokenStream, item: TokenStream) -> TokenStream {
    bitflags::bitflags(item)
}

use proc_macro::TokenStream;

use quote::quote;
use syn::DeriveInput;

pub fn bitflags(item: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(item).unwrap();

    // Saved for use in ['quote!()'] later
    let flag_name = input.ident;
    let flag_vis = input.vis;

    // Only work with enums for now at least
    let enum_data = match input.data {
        syn::Data::Enum(e) => e,
        _ => panic!("Only works with enums"),
    };

    // Calculate the smallest integer size to use.
    let total_variants = enum_data.variants.len();
    let next_power_of_2 = round_to_next_data_size(total_variants, 8);
    assert!(
        next_power_of_2 > 128,
        "Bitflags cannot be larger than 128 options currently"
    );

    let data_type = quote::format_ident!("u{next_power_of_2}");

    // Generate the flags
    let flags = enum_data.variants.iter().enumerate().map(|(idx, flag)| {
        let flag_ident = &flag.ident;
        quote! {
            const #flag_ident: #flag_name = #flag_name(1 << #idx);
        }
    });

    // Put it all together.
    let bitflags = quote!(
        #[derive(Eq, PartialEq)]
        #flag_vis struct #flag_name(#data_type);

        impl #flag_name {
            #(#flags)*
        }

        // Or - FLAG1 | FLAG2
        impl std::ops::BitOr for #flag_name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0.bitor(rhs.0))
            }
        }

        // Or - Flags |= FLAG2
        impl std::ops::BitOrAssign for #flag_name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0.bitor_assign(rhs.0)
            }
        }

        // And - Flags & Flag2 results in bool if flag is active
        impl std::ops::BitAnd for #flag_name {
            type Output = bool;

            fn bitand(self, rhs: Self) -> Self::Output {
                self.0.bitand(rhs.0) != 0
            }
        }


        // TODO: Improve later.
        impl std::fmt::Debug for #flag_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:#b}", self.0)
            }
        }
    );

    bitflags.into()
}

/// Rounds to the next whole stride value.
fn round_to_next_data_size(num: usize, mut stride: usize) -> usize {
    while stride < num {
        stride <<= 1;
    }
    stride
}

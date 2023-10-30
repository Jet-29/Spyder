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
        next_power_of_2 <= 128,
        "Bitflags cannot be larger than 128 options currently, You have {total_variants} rounded to {next_power_of_2}"
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

            fn bits(&self) -> #data_type {
                self.0
            }
        }

        // Or - FLAG1 | FLAG2
        impl std::ops::BitOr for #flag_name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.bits() | (rhs.bits()))
            }
        }

        // Or - Flags |= FLAG2
        impl std::ops::BitOrAssign for #flag_name {
            fn bitor_assign(&mut self, rhs: Self) {
                *self = Self(self.bits() | rhs.bits())
            }
        }

        // Symmetric difference
        impl std::ops::BitXor for #flag_name {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.bits() ^ rhs.bits())
            }
        }

        // Toggle
        impl std::ops::BitXorAssign for #flag_name {
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = Self(self.bits() ^ rhs.bits())
            }
        }


        // And
        impl std::ops::BitAnd for #flag_name {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.bits() & rhs.bits())
            }
        }

        impl std::ops::BitAndAssign for #flag_name {
            fn bitand_assign(&mut self, rhs: Self) {
                *self = Self(self.bits() & rhs.bits())
            }
        }

        // Difference
        impl std::ops::Sub for #flag_name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.bits() & !rhs.bits())
            }
        }

        impl std::ops::SubAssign for #flag_name {
            fn sub_assign(&mut self, rhs: Self) {
                *self = Self(self.bits() & !rhs.bits())
            }
        }

        impl std::ops::Not for #flag_name {
            type Output = Self;

            fn not(self) -> Self::Output {
                Self(!self.bits())
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

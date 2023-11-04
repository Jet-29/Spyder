use proc_macro::TokenStream;

use quote::quote;
use syn::DeriveInput;

pub fn bitflags(item: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(item).unwrap();

    // Saved for use in ['quote!()'] later
    let bitflags_name = input.ident;
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

    let flag_names = enum_data.variants.iter().map(|flag| &flag.ident);

    // Generate the flags
    let flags = flag_names.clone().enumerate().map(|(idx, name)| {
        quote! {
            pub const #name: #bitflags_name = #bitflags_name(1 << #idx);
        }
    });

    // Put it all together.
    let bitflags = quote!(
        #[derive(Eq, PartialEq)]
        #flag_vis struct #bitflags_name(#data_type);

        impl #bitflags_name {
            const EMPTY: #bitflags_name = #bitflags_name(0);
            #(#flags)*
            const FULL: #bitflags_name = #bitflags_name((1 << #total_variants) - 1);

            #[inline]
            pub const fn bits(&self) -> #data_type {
                self.0
            }

            #[inline]
            pub const fn from_bits(bits: #data_type) -> Self {
                Self(bits & Self::FULL.bits())
            }

            #[inline]
            pub fn is_empty(&self) -> bool {
                *self == Self::EMPTY
            }

            #[inline]
            pub fn is_full(&self) -> bool {
                *self == Self::FULL
            }

            #[inline]
            pub fn intersects(&self, other: Self) -> bool {
                !self.intersection(other).is_empty()
            }

            // TODO: Either add copy or clone, or find better way
            #[inline]
            pub fn contains_all(&self, other: Self) -> bool {
                let other_bits = other.bits();
                self.intersection(other).bits() == other_bits
            }

            #[inline]
            pub fn insert(&mut self, other: Self) {
                *self = self.union(other);
            }

            #[inline]
            pub fn keep_intersection(&mut self, other: Self) {
                *self = self.intersection(other);
            }

            #[inline]
            pub fn remove(&mut self, other: Self) {
                *self = self.difference(other);
            }

            #[inline]
            pub fn toggle(&mut self, other: Self) {
                *self = self.symetric_difference(other);
            }

            #[inline]
            pub fn set(&mut self, other: Self, value: bool) {
                if value {
                    self.insert(other);
                } else {
                    self.remove(other);
                }
            }

            #[inline]
            pub fn intersection(&self, other: Self) -> Self {
                Self(self.bits() & other.bits())
            }

            #[inline]
            pub fn union(&self, other: Self) -> Self {
                Self(self.bits() | other.bits())
            }

            #[inline]
            pub fn difference(&self, other: Self) -> Self {
                Self(self.bits() & !other.bits())
            }

            #[inline]
            pub fn symetric_difference(&self, other: Self) -> Self {
                Self(self.bits() ^ other.bits())
            }

            #[inline]
            pub fn compliment(&self) -> Self {
                Self::from_bits(!self.bits())
            }

        }

        impl std::ops::BitOr for #bitflags_name {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                self.union(rhs)
            }
        }

        impl std::ops::BitOrAssign for #bitflags_name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.insert(rhs)
            }
        }

        impl std::ops::BitXor for #bitflags_name {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                self.symetric_difference(rhs)
            }
        }

        impl std::ops::BitXorAssign for #bitflags_name {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.toggle(rhs)
            }
        }

        impl std::ops::BitAnd for #bitflags_name {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                self.intersection(rhs)
            }
        }

        impl std::ops::BitAndAssign for #bitflags_name {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.keep_intersection(rhs)
            }
        }

        impl std::ops::Sub for #bitflags_name {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                self.difference(rhs)
            }
        }

        impl std::ops::SubAssign for #bitflags_name {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                self.remove(rhs)
            }
        }

        impl std::ops::Not for #bitflags_name {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                self.compliment()
            }
        }

        impl std::fmt::Binary for #bitflags_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:#b}", self.0)
            }
        }

        // TODO: Improve later.
        impl std::fmt::Debug for #bitflags_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:#b}", self.0)
            }
        }

        impl std::fmt::Display for #bitflags_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut first = true;
                let mut bits = self.bits();
                while bits != 0 {
                    let bit = bits & (!bits + 1);
                    bits ^= bit;
                    if !first {
                        write!(f, " | ")?;
                    } else {
                        first = false;
                    }
                    match #bitflags_name::from_bits(bit) {
                        #(
                            #bitflags_name::#flag_names => write!(f, stringify!(#flag_names))?,
                        )*
                        _ => unreachable!(),
                    }
                }
                Ok(())
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

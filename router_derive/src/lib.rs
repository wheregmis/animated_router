use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn transition(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // Generate the trait
    let route_transition_trait = quote! {
        pub trait RouteTransition {
            fn get_transition(&self) -> TransitionVariant;
        }
    };

    // Generate the implementation
    let implementation = quote! {
        impl RouteTransition for #name {
            fn get_transition(&self) -> TransitionVariant {
                match self {
                    Self::Home {} => TransitionVariant::Fade,
                    Self::SlideLeft {} => TransitionVariant::SlideLeft,
                    Self::SlideRight {} => TransitionVariant::SlideRight,
                    Self::SlideUp {} => TransitionVariant::SlideUp,
                    Self::SlideDown {} => TransitionVariant::SlideDown,
                    Self::Fade {} => TransitionVariant::Fade,
                    Self::Scale {} => TransitionVariant::Scale,
                    _ => TransitionVariant::Fade,
                }
            }
        }
    };

    // Combine the original enum with the new implementations
    let expanded = quote! {
        #route_transition_trait
        #input
        #implementation
    };

    expanded.into()
}

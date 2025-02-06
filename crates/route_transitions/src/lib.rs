use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, Data, DataEnum, DeriveInput, Fields, Meta};

fn get_transition_from_attrs(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("transition"))
        .and_then(|attr| {
            if let Ok(Meta::Path(path)) = attr.parse_args::<Meta>() {
                path.get_ident().map(|ident| ident.to_string())
            } else {
                None
            }
        })
}

#[proc_macro_derive(RouteTransitions, attributes(transition))]
pub fn derive_route_transitions(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("RouteTransitions can only be derived for enums"),
    };

    let transition_match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let transition = get_transition_from_attrs(&variant.attrs)
            .map(|t| format_ident!("{}", t))
            .unwrap_or(format_ident!("Fade"));

        let pattern = match &variant.fields {
            Fields::Named(named_fields) => {
                let field_patterns = named_fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    quote! { #field_name }
                });
                quote! { Self::#variant_ident { #(#field_patterns,)* } }
            }
            Fields::Unnamed(_) => quote! { Self::#variant_ident(..) },
            Fields::Unit => quote! { Self::#variant_ident },
        };
        quote! {
            #pattern => TransitionVariant::#transition
        }
    });

    let component_match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let component_name = &variant.ident;

        if variant_ident == "PageNotFound" {
            let pattern = match &variant.fields {
                Fields::Named(named_fields) => {
                    let field_patterns = named_fields.named.iter().map(|f| {
                        let field_name = &f.ident;
                        quote! { #field_name }
                    });
                    quote! { Self::#variant_ident { #(#field_patterns,)* } }
                }
                _ => panic!("PageNotFound must have named fields"),
            };

            quote! {
                #pattern => (|| {
                    let props = PageNotFoundProps { route: route.clone() };
                    PageNotFound(props)
                })()
            }
        } else {
            let pattern = match &variant.fields {
                Fields::Named(_) => quote! { Self::#variant_ident { .. } },
                Fields::Unnamed(_) => quote! { Self::#variant_ident(..) },
                Fields::Unit => quote! { Self::#variant_ident },
            };

            quote! {
                #pattern => #component_name()
            }
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn get_transition(&self) -> TransitionVariant {
                match self {
                    #(#transition_match_arms,)*
                }
            }

            pub fn get_component(&self) -> Result<VNode, RenderError> {
                match self {
                    #(#component_match_arms,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

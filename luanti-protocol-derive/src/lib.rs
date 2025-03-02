//! Derive macros for luanti-protocol

#![expect(
    missing_docs,
    // clippy::missing_panics_doc,
    // clippy::missing_errors_doc,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unimplemented,
    reason = "//TODO add documentation and improve error handling"
)]

use proc_macro2::Ident;
use proc_macro2::Literal;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use quote::quote_spanned;
use syn::Data;
use syn::DeriveInput;
use syn::Field;
use syn::Generics;
use syn::Index;
use syn::Type;
use syn::TypeParam;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

#[proc_macro_derive(LuantiSerialize, attributes(wrap))]
pub fn luanti_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let serialize_body = make_serialize_body(&name, &input.data);

    // The struct must include Serialize in the bounds of any type
    // that need to be serializable.
    let impl_generic = input.generics.to_token_stream();
    let name_generic = strip_generic_bounds(&input.generics).to_token_stream();
    let where_generic = input.generics.where_clause;

    let expanded = quote! {
        impl #impl_generic Serialize for #name #name_generic #where_generic {
            type Input = Self;
            fn serialize<S: Serializer>(value: &Self::Input, ser: &mut S) -> SerializeResult {
                #serialize_body
                Ok(())
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(LuantiDeserialize, attributes(wrap))]
pub fn luanti_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let deserialize_body = make_deserialize_body(&name, &input.data);

    // The struct must include Deserialize in the bounds of any type
    // that need to be serializable.
    let impl_generic = input.generics.to_token_stream();
    let name_generic = strip_generic_bounds(&input.generics).to_token_stream();
    let where_generic = input.generics.where_clause;

    let expanded = quote! {
        impl #impl_generic Deserialize for #name #name_generic #where_generic {
            type Output = Self;
            fn deserialize(deser: &mut Deserializer) -> DeserializeResult<Self> {
                #deserialize_body
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn get_wrapped_type(field: &Field) -> Type {
    let mut ty = field.ty.clone();
    for attr in &field.attrs {
        if attr.path().is_ident("wrap") {
            ty = attr.parse_args::<Type>().unwrap();
        }
    }
    ty
}

/// For struct, fields are serialized/deserialized in order.
/// For enum, tags are assumed u8, consecutive, starting with 0.
fn make_serialize_body(input_name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|field| {
                    let name = &field.ident;
                    let ty = get_wrapped_type(field);
                    quote_spanned! {field.span() =>
                        <#ty as Serialize>::serialize(&value.#name, ser)?;
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            syn::Fields::Unnamed(ref fields) => {
                let recurse = fields.unnamed.iter().enumerate().map(|(index, field)| {
                    let index = Index::from(index);
                    let ty = get_wrapped_type(field);
                    quote_spanned! {field.span() =>
                        <#ty as Serialize>::serialize(&value.#index, ser)?;
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            syn::Fields::Unit => {
                quote! {}
            }
        },
        Data::Enum(ref body) => {
            let recurse = body.variants.iter().enumerate().map(|(index, variant)| {
                if !variant.fields.is_empty() {
                    quote_spanned! {variant.span() =>
                        compile_error!("Cannot handle fields yet");
                    }
                } else if variant.discriminant.is_some() {
                    quote_spanned! {variant.span() =>
                        compile_error!("Cannot handle discrimiant yet");
                    }
                } else {
                    let id = &variant.ident;
                    let i = Literal::u8_unsuffixed(
                        u8::try_from(index).expect("variant index exceeds range of u8"),
                    );
                    quote_spanned! {variant.span() =>
                        #id => #i,
                    }
                }
            });
            quote! {
                    use #input_name::*;
                    let tag = match value {
                        #(#recurse)*
                    };
                    u8::serialize(&tag, ser)?;
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}

fn make_deserialize_body(input_name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            let inner = match data.fields {
                syn::Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|field| {
                        let name = &field.ident;
                        let ty = get_wrapped_type(field);
                        quote_spanned! {field.span() =>
                            #name: <#ty as Deserialize>::deserialize(deser)?,
                        }
                    });
                    quote! {
                        #(#recurse)*
                    }
                }
                syn::Fields::Unnamed(ref fields) => {
                    let recurse = fields.unnamed.iter().enumerate().map(|(index, field)| {
                        let index = Index::from(index);
                        let ty = get_wrapped_type(field);
                        quote_spanned! {field.span() =>
                            #index: <#ty as Deserialize>::deserialize(deser)?,
                        }
                    });
                    quote! {
                        #(#recurse)*
                    }
                }
                syn::Fields::Unit => {
                    quote! {}
                }
            };
            quote! {
                Ok(Self {
                    #inner
                })
            }
        }
        Data::Enum(ref body) => {
            let recurse = body.variants.iter().enumerate().map(|(index, variant)| {
                if !variant.fields.is_empty() {
                    quote_spanned! {variant.span() =>
                        compile_error!("Cannot handle fields yet");
                    }
                } else if variant.discriminant.is_some() {
                    quote_spanned! {variant.span() =>
                        compile_error!("Cannot handle discriminant yet");
                    }
                } else {
                    let id = &variant.ident;
                    let i = Literal::u8_unsuffixed(
                        u8::try_from(index).expect("variant index exceeds range of u8"),
                    );
                    quote_spanned! {variant.span() =>
                        #i => #id,

                    }
                }
            });

            let input_name_str = Literal::string(&input_name.to_string());
            quote! {
                    use #input_name::*;
                    let tag = u8::deserialize(deser)?;
                    Ok(match tag {
                        #(#recurse)*
                        _ => bail!("Invalid {} tag: {}", #input_name_str, tag),
                    })
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}

/// Converts <T: Trait, S: Trait2> into <T, S>
fn strip_generic_bounds(input: &Generics) -> Generics {
    let input = input.clone();
    Generics {
        lt_token: input.lt_token,
        params: {
            let mut params = input.params.clone();
            params.iter_mut().for_each(|param| {
                *param = match param.clone() {
                    syn::GenericParam::Type(param) => syn::GenericParam::Type(TypeParam {
                        attrs: Vec::new(),
                        ident: param.ident.clone(),
                        colon_token: None,
                        bounds: Punctuated::new(),
                        eq_token: None,
                        default: None,
                    }),
                    any => any,
                }
            });
            params
        },
        gt_token: input.gt_token,
        where_clause: None,
    }
}

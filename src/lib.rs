use syn::{braced, bracketed, parse::Parse, punctuated::Punctuated, token::Comma, Ident, LitInt};

use syn::token;

use proc_macro::TokenStream;

use quote::quote;

struct Field {
    name: Ident,
    typ: Ident,
    arr: Option<LitInt>,
}

struct Struct {
    name: Ident,
    fields: Punctuated<Field, Comma>,
}

struct Structs {
    structs: Vec<Struct>,
}

impl Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // type
        let typ: Ident = input.parse()?;

        let mut arr = None;

        if input.peek(token::Bracket) {
            let content;

            bracketed!(content in input);

            let size: syn::LitInt = content.parse()?;

            arr = Some(size);
        }

        let ident: Ident = input.parse()?;

        if input.peek(token::Bracket) {
            let content;

            bracketed!(content in input);

            let size: syn::LitInt = content.parse()?;

            arr = Some(size);
        }

        Ok(Self {
            typ,
            name: ident,
            arr,
        })
    }
}

impl Parse for Struct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<token::Struct>()?;

        let name = input.parse::<Ident>()?;

        let content;

        braced!(content in input);

        let mut fields = Punctuated::new();

        loop {
            if content.is_empty() {
                break;
            }

            let field: Field = content.parse()?;

            content.parse::<syn::token::Semi>()?;

            fields.push(field);
        }

        input.parse::<syn::token::Semi>()?;

        Ok(Self { name, fields })
    }
}

impl Parse for Structs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut structs: Vec<Struct> = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }

            let c: Struct = input.parse()?;

            structs.push(c);
        }

        Ok(Structs { structs })
    }
}

#[proc_macro]
pub fn c2rs_def(input: TokenStream) -> TokenStream {
    let structs = syn::parse::<Structs>(input).expect("error");

    let mut rs_structs = Vec::new();

    for c in structs.structs {
        let name = c.name;
        let fields = c.fields;
        let mut rs_fields = Vec::new();

        for field in fields {
            let name = field.name;
            let typ = field.typ;
            let typ = {
                if let Some(arr) = field.arr {
                    quote! {
                        [#typ; #arr]
                    }
                } else {
                    quote! {
                        #typ
                    }
                }
            };

            rs_fields.push(quote! {
                pub #name: #typ
            })
        }

        rs_structs.push(quote! {
            #[allow(unused)]
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            #[derive(Debug, Clone, Default)]
            #[repr(C)]
            pub struct #name{
                #(#rs_fields), *
            }

            impl #name{

                #[inline]
                pub unsafe fn from_mut_bytes(bytes: *mut u8) -> *mut Self{
                    std::mem::transmute(bytes)
                }

                #[inline]
                pub unsafe fn from_bytes(bytes: *const u8) -> *const Self{
                    std::mem::transmute(bytes)
                }

                #[inline]
                pub fn size() -> usize{
                    std::mem::size_of::<Self>()
                }

            }

        })
    }

    let gen = quote! {
        #(#rs_structs)*
    };

    gen.into()
}

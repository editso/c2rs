use syn::{braced, bracketed, parse::Parse, token::Semi, Ident, LitInt};

use syn::token;

use proc_macro::TokenStream;

use quote::quote;

enum Type {
    /// union { ... }
    Union(Struct),
    /// struct { ... }
    Struct(Struct),
    /// ident
    Ident(Ident),
    /// count, ident
    Ptr(Vec<token::Star>, Ident),
    /// []
    Array(Ident, Box<Type>),
    /// number
    Number(LitInt),
}

struct Field {
    name: Option<Ident>,
    typ: Type,
}

struct Fields {
    fields: Vec<Field>,
}

struct Struct {
    name: Ident,
    fields: Fields,
}

struct Structs {
    structs: Vec<Type>,
}

impl Parse for Structs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut structs = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }

            let c = input.parse()?;

            input.parse::<token::Semi>()?;

            structs.push(c);
        }

        Ok(Structs { structs })
    }
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(token::Struct) {
            Self::parse_define_struct(input)
        } else if input.peek(token::Union) {
            Self::parse_define_union(input)
        } else {
            Err(input.error("expect struct or union"))
        }
    }
}

impl Type {
    fn parse_define_struct(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<token::Struct>()?;

        Ok(Self::Struct(Struct {
            name: input.parse::<Ident>()?,
            fields: {
                let content;

                braced!(content in input);

                content.parse::<Fields>()?
            },
        }))
    }

    fn parse_define_union(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<token::Union>()?;

        Ok(Self::Union(Struct {
            name: input.parse::<Ident>()?,
            fields: {
                let content;

                braced!(content in input);

                content.parse::<Fields>()?
            },
        }))
    }

    fn parse_struct(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<token::Struct>()?;
        let content;

        braced!(content in input);

        content.parse::<Fields>()?;

        let fields = content.parse::<Fields>()?;

        Ok(Self::Struct(Struct {
            name: input.parse()?,
            fields,
        }))
    }

    fn parse_union(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<token::Union>()?;
        let content;

        braced!(content in input);

        let fields = content.parse::<Fields>()?;

        Ok(Self::Union(Struct {
            name: input.parse()?,
            fields,
        }))
    }

    fn parse_ptr(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut stars = vec![input.parse::<token::Star>()?];

        loop {
            if input.peek(token::Star) {
                stars.push(input.parse::<token::Star>()?);
            } else {
                break;
            }
        }

        Ok(Self::Ptr(stars, input.parse::<Ident>()?))
    }

    fn parse_array(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        bracketed!(content in input);

        let typ = if content.peek(LitInt) {
            Self::Number(content.parse()?)
        } else if content.peek(Ident) {
            Self::Ident(content.parse()?)
        } else {
            return Err(content.error(format!(
                "expect usize or ident found {}",
                content.to_string()
            )));
        };

        if !content.is_empty() {
            return Err(content.error(format!("syntax error: {}", content.to_string())));
        }

        Ok(typ)
    }

    fn parse_ident(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self::Ident(input.parse()?))
    }
}

impl Parse for Fields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut fields = Vec::new();

        loop {
            if input.is_empty() {
                break;
            }

            let field = input.parse::<Field>()?;

            input.parse::<Semi>()?;

            fields.push(field);
        }

        Ok(Fields { fields })
    }
}

impl Parse for Field {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let typ = if input.peek(token::Union) {
            Type::parse_union(input)?
        } else if input.peek(token::Struct) {
            Type::parse_struct(input)?
        } else {
            let typ = input.parse::<Ident>()?;

            let mut name = if input.peek(token::Bracket) {
                // [] name
                let arr = Type::parse_array(input)?;
                let name = input.parse::<Ident>()?;
                Type::Array(name, Box::new(arr))
            } else if input.peek(token::Star) {
                Type::parse_ptr(input)?
            } else {
                Type::parse_ident(input)?
            };

            if input.peek(token::Bracket) {
                // name[]
                name = match name {
                    Type::Ident(ident) => Type::Array(ident, Box::new(Type::parse_array(input)?)),
                    _ => return Err(input.error("syntax error")),
                }
            }

            return Ok(Field {
                name: Some(typ),
                typ: name,
            });
        };

        Ok(Field { name: None, typ })
    }
}


/// C struct to Rust struct 
/// #Example
/// ```
/// use c2rs::c2rs_def;
/// 
/// type DWORD = u32;
/// const SIZE: usize = 10;
/// c2rs_def!(
///     struct A{
///         DWORD var1;
///         DWORD var2;
///         union {
///              DWORD var4;
///              DWORD var5;   
///         }var3;
///         
///         struct {
///             u8 var7;
///         }var6;
/// 
///         DWORD array[SIZE];
///     };
///     
///     struct B{
///         u8 var1;
///     };
///     
///     // ....
/// );
/// 
/// let mut buffer = [1u8; 1024];
/// 
/// unsafe{
///     let mut buf = A::from_mut_bytes(buffer.as_mut_ptr());
///     let buf = buf.as_mut().unwrap();
///     buf.var1 = 10;
///     
///     assert_eq!(10, buf.var1);
///     assert_eq!(10, buffer[0]);
///     
///     let mut b = B::from_mut_bytes(buffer.as_mut_ptr()).as_mut().unwrap();
///     
///     assert_eq!(10, b.var1);
/// 
/// }
/// 
/// ```
#[proc_macro]
pub fn c2rs_def(input: TokenStream) -> TokenStream {
    let structs = syn::parse::<Structs>(input).expect("error");

    let mut structs = structs.structs;

    let mut rs_structs = Vec::new();

    loop {
        let c = structs.pop();

        if c.is_none() {
            break;
        }

        let c = c.unwrap();

        let (type_decl, name, fields, is_union, derives) = match c {
            Type::Union(u) => (quote! { union }, u.name, u.fields.fields, true, quote! {}),
            Type::Struct(s) => (
                quote! { struct},
                s.name,
                s.fields.fields,
                false,
                quote! {
                    #[derive(Debug)]
                },
            ),
            _ => panic!("syntax error"),
        };

        let mut rs_fields = Vec::new();

        for field in fields {
            if let Some(name) = field.name {
                let token = match field.typ {
                    Type::Ident(typ) => {
                        quote! {
                            pub #typ: #name
                        }
                    }
                    Type::Array(typ, array) => match *array {
                        Type::Ident(ident) => quote! {
                            pub #typ: [#name; #ident]
                        },
                        Type::Number(n) => quote! {
                            pub #typ: [#name; #n]
                        },
                        _ => panic!("syntax error"),
                    },
                    Type::Ptr(stars, typ) => {
                        quote! {
                            pub #typ: #(#stars mut)* #name
                        }
                    }
                    _ => panic!("syntax error"),
                };

                rs_fields.push(token);

                continue;
            }

            let (typ, token) = match field.typ {
                Type::Struct(mut u) => {
                    let field_name = u.name.clone();
                    let typ = syn::parse_str::<Ident>(&format!(
                        "{}_{}",
                        name.to_string(),
                        field_name.to_string().to_uppercase()
                    ))
                    .unwrap();

                    u.name = typ.clone();

                    (
                        Type::Struct(u),
                        quote! {
                            pub #field_name: #typ
                        },
                    )
                }
                Type::Union(mut u) => {
                    let field_name = u.name;
                    let typ = syn::parse_str::<Ident>(&format!(
                        "{}_{}",
                        name.to_string(),
                        field_name.to_string().to_uppercase()
                    ))
                    .unwrap();

                    u.name = typ.clone();

                    (
                        Type::Union(u),
                        quote! {
                            pub #field_name: #typ
                        },
                    )
                }
                _ => panic!("syntax error"),
            };

            structs.push(typ);
            rs_fields.push(token);
        }

        let impl_traits = if is_union {
            let stringify = name.to_string();
            quote! {

                impl std::fmt::Debug for #name{
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
                        writeln!(f, "union<{}>", #stringify)
                    }
                }

            }
        } else {
            quote! {}
        };

        rs_structs.push(quote! {
            #[allow(unused)]
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            #derives
            #[repr(C)]
            pub #type_decl #name{
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

            #impl_traits

        })
    }

    let gen = quote! {
        #(#rs_structs)*
    };

    gen.into()
}

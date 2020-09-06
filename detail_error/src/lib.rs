use proc_macro::{TokenStream};
use syn::{parse_macro_input, parse2, DeriveInput};
use proc_macro2::{Ident, Span};
use quote::quote;
use darling::{FromDeriveInput, FromVariant, FromMeta, FromField};

use darling::{ast, util};
use syn::{Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(detail), supports(enum_any))]
struct DetailErrorEnum {
    ident: syn::Ident,
    data: darling::ast::Data<DetailErrorVariant, darling::util::Ignored>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(detail), supports(struct_any))]
struct DetailErrorStruct {
    ident: syn::Ident,
    // LoremField用于提取所有的字段
    data: darling::ast::Data<DetailErrorVariant_, LoremField>,
}

#[derive(Debug, FromField)]
#[darling(attributes(lorem))]
pub struct LoremField {
    // 提取字段名
    ident: Option<Ident>,
    // 提取字段类型
    ty: Type,
    // 可在目标结构字段的标注上使用的属性
    // ref: https://github.com/TedDriggs/darling/blob/master/examples/supports_struct.rs
    #[darling(default)]
    skip: bool,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(detail))]
struct DetailErrorVariant {
    ident: syn::Ident,
    fields: darling::ast::Fields<syn::Field>,
    #[darling(default)]
    code: Option<u16>,
    #[darling(default)]
    message: Option<String>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(detail))]
struct DetailErrorVariant_ {
    ident: syn::Ident,
    fields: darling::ast::Fields<syn::Field>,
    #[darling(default)]
    code: Option<u16>,
    #[darling(default)]
    message: Option<String>,
}

// #[derive(Debug, FromField)]
// #[darling(attributes(lorem))]
// pub struct LoremField {
//     ident: Option<Ident>,
//     ty: Type,
//     #[darling(default)]
//     skip: bool,
// }
//
// #[derive(Debug, FromDeriveInput)]
// #[darling(attributes(lorem), supports(struct_named))]
// pub struct Lorem {
//     ident: Ident,
//     data: ast::Data<util::Ignored, LoremField>,
// }

#[proc_macro_derive(DetailError, attributes(detail))]
pub fn detail_error_fn(input: TokenStream) -> TokenStream {
    handler(input.into()).into()
}

fn handler(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let result = parse2::<DeriveInput>(input).unwrap();
    let detail_error: DetailErrorEnum = DetailErrorEnum::from_derive_input(&result).unwrap();

    dbg!(&detail_error);

    let ident = &detail_error.ident;
    let variants = detail_error.data.take_enum().unwrap();
    let http_code_fn_codegen: Vec<proc_macro2::TokenStream> = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;

        let http_code = variant.code.unwrap_or(400);

        quote! {
            #ident::#variant_ident => #http_code
        }
    }).collect();
    let code_fn_codegen: Vec<proc_macro2::TokenStream> = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let content = inflector::cases::screamingsnakecase::to_screaming_snake_case(&variant_ident.to_string());
        quote! {
            #ident::#variant_ident => String::from(#content)
        }
    }).collect();

    let message_fn_codegen: Vec<proc_macro2::TokenStream> = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let message = variant.message.clone().unwrap_or_else(|| {
            inflector::cases::sentencecase::to_sentence_case(&variant_ident.to_string())
        });
        quote! {
            #ident::#variant_ident => String::from(#message)
        }
    }).collect();

    let output = quote! {
        impl #ident {
            pub fn get_http_code(&self) -> u16 {
                match self {
                    #(#http_code_fn_codegen,)*
                }
            }
            pub fn get_code(&self) -> String {
                match self {
                    #(#code_fn_codegen,)*
                }
            }
            pub fn get_message(&self) -> String {
                match self {
                    #(#message_fn_codegen,)*
                }
            }
        }
    };
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler;
    use quote::quote;

    #[test]
    fn it_works() {
        let input = quote! {
            pub enum A {
                A,

            }
        };
        let expected_output = quote! {
            impl A {
                pub fn get_http_code(&self) -> u16 {
                    match self {
                        A::A => 400u16,
                    }
                }
                pub fn get_code(&self) -> String {
                    match self {
                        A::A => String::from("A"),
                    }
                }
                pub fn get_message(&self) -> String {
                    match self {
                        A::A => String::from("A"),
                    }
                }
            }
        };
        let output = handler(input);
        assert_eq!(expected_output.to_string(), output.to_string());
    }

    #[test]
    fn it_enum_output() {
        // let input = quote! {
        //     pub struct ShippingEvent<Moment> {
        //         id: ShippingEventId,
        //         event_type: ShippingEventType,
        //         shipment_id: ShipmentId,
        //         location: Option<ReadPoint>,
        //         readings: Vec<Reading<Moment>>,
        //         timestamp: Moment,
        //     }
        // };
        let input = quote! {
            pub enum A {
                A,

            }
        };
        println!("{:?}", input.to_string());
        // let input = quote! {
        //     <item>value</item>
        // };
        // println!("{:?}", input.to_string());
        let result = parse2::<DeriveInput>(input).unwrap();
        // dbg!(&result);
        // let detail_error: Lorem = Lorem::from_derive_input(&result).unwrap();
        let detail_error: DetailErrorEnum = DetailErrorEnum::from_derive_input(&result).unwrap();
        dbg!(&detail_error);
    }

    #[test]
    fn it_struct_output() {
        let input = quote! {
            pub struct ShippingEvent<Moment> {
                id: ShippingEventId,
                event_type: ShippingEventType,
                shipment_id: ShipmentId,
                location: Option<ReadPoint>,
                #[lorem(skip)]
                readings: Vec<Reading<Moment>>,
                timestamp: Moment,
            }
        };
        // let input = quote! {
        //     pub enum A {
        //         A,
        //
        //     }
        // };
        println!("{:?}", input.to_string());
        // let input = quote! {
        //     <item>value</item>
        // };
        // println!("{:?}", input.to_string());
        let result = parse2::<DeriveInput>(input).unwrap();
        dbg!(&result);

        let detail_error: DetailErrorStruct = DetailErrorStruct::from_derive_input(&result).unwrap();
        dbg!(&detail_error);
    }

    // #[test]
    // fn it_xml_output() {
    //     let input = quote! {
    //         <item>value</item>
    //     };
    //     println!("{:?}", input.to_string());
    //     let result = parse2::<DeriveInput>(input).unwrap();
    //     dbg!(&result);
    // }

}

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

/// ### Rust 编译器对过程宏的定义有以下限制：
///
/// - 过程宏函数（带有 #[proc_macro_derive] 或其他类似属性的函数）必须位于 crate 根模块。
/// - 实现细节可以拆分到其他模块，但过程宏的入口函数需要直接暴露在 crate 根部。
///

mod builder;
mod service;


/// 创建一个 #[service] 宏，用来为每个结构体提供标记，并且自动将它们注册到全局的服务列表中
// #[proc_macro_attribute]
// pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(item as syn::Item);
//
//     match input.clone() {
//         syn::Item::Struct(s) => service::impl_service_for_struct(s, input).into(),
//         // syn::Item::Impl(i) => service::impl_service_for_impl(i, input).into(),
//         _ => {
//             let error = syn::Error::new_spanned(
//                 input,
//                 "The `service` attribute can only be applied to structs"
//             );
//             error.to_compile_error().into()
//         }
//     }
// }

// #[proc_macro_attribute]
// pub fn service(_args: TokenStream, item: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(item as ItemStruct);
//     service::register_services(input).into()
// }

#[proc_macro_attribute]
pub fn service(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // 解析输入的 TokenStream 为结构体的 AST
    let input = parse_macro_input!(input as Item);
    match input.clone() {
        Item::Struct(s) => {
            // 获取结构体的名称
            let struct_name = &s.ident;
            // 生成代码：inventory::submit!(...)
            let expanded = quote! {
                // 保留原始结构体定义
                #s

                // 自动生成 inventory::submit! 注册代码
                inventory::submit!(&#struct_name as &dyn WebService);
            };
            // 返回生成的代码
            expanded.into()
        }
        _ => {
            let error = syn::Error::new_spanned(
                input,
                "The `service` attribute can only be applied to structs",
            );
            error.to_compile_error().into()
        }
    }
}

/// ## 实现 #[builder] 宏，生成构建器模式代码：
///
/// Implements the builder pattern for a struct, with optional getter and setter methods.
/// # Field Attributes
///
/// - `#[builder(getter)]`: Generates a getter method for the field
/// - `#[builder(setter)]`: Generates a setter method for the field
/// - `#[builder(getter, setter)]`: Generates both getter and setter methods
///
/// # Example
///
/// ```ignore
/// use macros::builder;
///
/// #[builder]
/// struct Service {
///     #[builder(getter)]
///     name: String,
///     #[builder(setter)]
///     count: i32,
///     #[builder(getter, setter)]
///     enabled: bool,
/// }
///
/// let mut service = Service::builder()
///     .name("test".to_string())
///     .count(42)
///     .enabled(true)
///     .build()
///     .unwrap();
///
/// // Use generated getter
/// assert_eq!(service.get_name(), &"test".to_string());
///
/// // Use generated setter
/// service.set_count(100);
/// service.set_enabled(false);
/// ```
// #[proc_macro_derive(Builder, attributes(builder))]
#[proc_macro_attribute]
pub fn builder(_attr: TokenStream, input: TokenStream) -> TokenStream {
    builder::builder_macro_impl(input)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::register_services;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn test_service() {
        let input = quote! {
            #[service]
            struct MyService;

        };

        let item = parse2(input).unwrap();

        println!("{:?}", register_services(item).to_string());
    }
}

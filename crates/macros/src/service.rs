use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemStruct;


#[allow(unused)]
pub fn register_services(s: ItemStruct) -> TokenStream {
    // 获取结构体的名称
    let struct_name = &s.ident;

    let expanded = quote! {

        // impl #struct_name {
            // pub fn register() {
            //     use web_core::web_service::register_service;
            //     let service = std::sync::Arc::new(#struct_name);
            //     register_service(service);
            // }
        // }
        inventory::submit!(&#struct_name as &dyn web_core::web_service::WebService);
    };

    TokenStream::from(expanded)
}

#[allow(unused)]
pub fn impl_service_for_struct(s: ItemStruct, input: syn::Item) -> TokenStream {
    let struct_name = &s.ident;
    let registration_fn_name = format_ident!("__inventory_register_{}", struct_name);

    let expanded = quote! {
        #input

        // 使用 inventory 注册服务的函数
        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #registration_fn_name() {

            inventory::submit!(ServiceRegistry {
                service: Box::new(#struct_name {})
            });
        }

        // 确保注册函数被调用
        // #[used]
        // #[cfg_attr(target_os = "linux", link_section = ".init_array")]
        // #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        // #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        // static #registration_fn_name: fn() = #registration_fn_name;
    };

    expanded.into()
}

#[allow(unused)]
pub fn _impl_service_for_impl(i: syn::ItemImpl, input: syn::Item) -> TokenStream {
    let type_name = &i.self_ty;
    let type_name_string = quote!(#type_name);
    let registration_fn_name = format_ident!("__inventory_register_{}", type_name_string.to_string());

    let expanded = quote! {
        #input

        // 使用 inventory 注册服务的函数
        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #registration_fn_name() {
            use std::sync::Arc;
            inventory::submit!(ServiceRegistry {
                service: Arc::new(#type_name {})
            });
        }

        // 确保注册函数被调用
        #[used]
        #[cfg_attr(target_os = "linux", link_section = ".init_array")]
        #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        static #registration_fn_name: fn() = #registration_fn_name;
    };

    expanded.into()
}
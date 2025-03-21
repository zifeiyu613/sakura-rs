use darling::FromField;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident};


#[derive(Debug, FromField)]
#[darling(attributes(builder))]
struct BuilderField {
    ident: Option<Ident>, // 字段名
    ty: syn::Type,        // 字段类型
    #[darling(default)]
    getter: bool, // 是否生成 getter
    #[darling(default)]
    setter: bool, // 是否生成 setter
}

pub fn builder_macro_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // 获取结构体名称、可见性和泛型
    let struct_name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // 获取字段信息
    let fields = if let Data::Struct(data) = &input.data {
        data.fields
            .iter()
            .map(BuilderField::from_field)
            .collect::<Result<Vec<_>, _>>()
    } else {
        panic!("#[Builder] only supports structs")
    }
    .unwrap();

    // 生成 Builder 名称
    let builder_name = Ident::new(&format!("{}Builder", struct_name), Span::call_site());

    // 生成构建器Builder字段
    let builder_fields_for_struct = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        quote! {
            #ident: Option<#ty>
        }
    });

    let builder_fields = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        // let ty = &field.ty;
        quote! {
            #ident: None
        }
    });

    // 生成 `build` 方法中的字段初始化和校验
    let build_fields = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        quote! {
            // #ident: self.#ident.take().expect(concat!("Field '", stringify!(#ident), "' is required"))
            #ident: self.#ident.ok_or_else(||errors::build_error::BuildError::MissingDependency(stringify!(#ident).to_string()))?
        }
    });

    // 为所有构建器字段生成赋值方法
    let builder_fields_methods = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        quote! {
            pub fn #ident(mut self, value: #ty) -> Self {
                self.#ident = Some(value);
                self
            }
        }
    });

    // 为结构体生成 `setter` 和 `getter` 方法
    let setter_methods = fields.iter().filter(|field| field.setter).map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let setter_name = Ident::new(
            &format!("set_{}", ident),
            Span::call_site(),
        );
        quote! {
            pub fn #setter_name(mut self, value: #ty) -> Self {
                self.#ident = value;
                self
            }
        }
    });

    let getter_methods = fields.iter().filter(|field| field.getter).map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let getter_name = Ident::new(
            &format!("get_{}", ident),
            Span::call_site(),
        );
        quote! {
            pub fn #getter_name(&self) -> &#ty {
                &self.#ident
            }
        }
    });

    // 生成完整代码
    let expanded = quote! {

        // 构建器定义
        #vis struct #builder_name #impl_generics #where_clause {
            #(#builder_fields_for_struct),*
        }

        impl #impl_generics #builder_name #ty_generics #where_clause {

            #(#builder_fields_methods)*

            // `build` 方法
            pub fn build(self) -> Result<#struct_name #ty_generics, errors::build_error::BuildError> {
                Ok(#struct_name {
                    #(#build_fields),*
                })
            }
        }

        // 为目标结构体生成 `builder` 方法
        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn builder() -> #builder_name #ty_generics {
                #builder_name {
                    #(#builder_fields),*
                }
            }

            // 生成 getter 方法
            #(#getter_methods)*
            // 生成 Setter 方法
            #(#setter_methods)*
        }

    };

    // 输出生成的代码到终端
    // std::fs::write("tests/builder_expanded.rs", format!("{}", expanded)).expect("Unable to write expanded code");

    expanded.into()
}

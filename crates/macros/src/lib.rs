use proc_macro::TokenStream;

/// ### Rust 编译器对过程宏的定义有以下限制：
///
/// - 过程宏函数（带有 #[proc_macro_derive] 或其他类似属性的函数）必须位于 crate 根模块。
/// - 实现细节可以拆分到其他模块，但过程宏的入口函数需要直接暴露在 crate 根部。
///

mod builder;

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
#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_macro(input: TokenStream) -> TokenStream {
    builder::builder_macro_impl(input)
}
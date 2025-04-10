pub mod request_dto;

pub use request_dto::*;

use crate::middleware::extract::NestedField;

// impl NestedField for OrderDTO {
//     fn field_name() -> &'static str {
//         "orderDTO"
//     }
// }
/// 使用宏简化上述代码
/// #[macro_export] 会自动将宏导出到 crate 根
#[macro_export]
macro_rules! impl_nested_field {
    ($struct_name:ty, $field_name:expr) => {
        impl NestedField for $struct_name {
            fn field_name() -> &'static str {
                $field_name
            }
        }
    };
}
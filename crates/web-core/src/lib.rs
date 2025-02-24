//! **Web 框架核心 (提供 WebService trait)**
//! - 它指定了 WebService trait 的 supertraits，也就是说，
//!    任何实现了 WebService trait 的类型，必须同时实现 Send 和 Sync 这两个 trait。
//! - Send trait: Send 是一个 marker trait，表示实现了 Send 的类型可以在线程之间安全地转移所有权。
//!    换句话说，一个 Send 类型的值可以安全地从一个线程移动到另一个线程。 大部分类型都是 Send，但有一些类型，
//!    比如 Rc (引用计数)，不是 Send，因为它们在多线程环境下不安全。
//! - Sync trait: Sync 也是一个 marker trait，表示实现了 Sync 的类型可以在多个线程之间安全地共享引用。
//!    也就是说，多个线程可以同时拥有一个 Sync 类型值的不可变引用 (&T)。 大多数类型都是 Sync，
//!    但如果一个类型包含内部可变性（例如，使用 Cell 或 RefCell），并且没有采取适当的同步措施，那么它可能不是 Sync。
//! - 为什么需要 Send + Sync？ 在 web 服务中，通常需要在多个线程中处理请求。
//!    Send + Sync 约束确保了任何实现了 WebService trait 的类型都可以在多线程环境中安全地使用。
//!    这对于并发处理请求至关重要，可以避免数据竞争和其他并发问题。

pub mod web_service;


// 使用 #[service] 代替
// #[macro_export]
// macro_rules! register_service {
//     ($ty:ident) => {
//         inventory::submit!(&$ty as &dyn WebService);
//     };
// }
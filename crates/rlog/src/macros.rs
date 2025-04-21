//! 便捷日志宏

/// 定义一个模块级日志记录器
///
/// 此宏创建一个模块级的记录器，方便对当前模块进行日志记录
#[macro_export]
macro_rules! module_logger {
    () => {
        const _: () = {
            use tracing as _tracing;
            
            #[allow(non_upper_case_globals)]
            static module_path: &str = module_path!();
            
            pub fn trace<S: std::fmt::Display>(message: S) {
                _tracing::trace!(target: module_path, "{}", message);
            }
            
            pub fn debug<S: std::fmt::Display>(message: S) {
                _tracing::debug!(target: module_path, "{}", message);
            }
            
            pub fn info<S: std::fmt::Display>(message: S) {
                _tracing::info!(target: module_path, "{}", message);
            }
            
            pub fn warn<S: std::fmt::Display>(message: S) {
                _tracing::warn!(target: module_path, "{}", message);
            }
            
            pub fn error<S: std::fmt::Display>(message: S) {
                _tracing::error!(target: module_path, "{}", message);
            }
        };
    };
}

/// 记录函数执行时间
///
/// 此宏会包装函数调用，并记录其执行时间
///
/// # 示例
/// ```
/// use rlog::time_it;
///
/// fn expensive_operation() -> i32 {
///     // Some expensive operation
///     42
/// }
///
/// fn main() {
///     let result = time_it!("expensive_operation", expensive_operation());
///     assert_eq!(result, 42);
/// }
/// ```
#[macro_export]
macro_rules! time_it {
    ($name:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::info!("{} completed in {:?}", $name, duration);
        result
    }};
}

/// 记录异步函数执行时间
///
/// 此宏会包装异步函数调用，并记录其执行时间
///
/// # 示例
/// ```
/// use rlog::time_it_async;
///
/// async fn expensive_async_operation() -> i32 {
///     // Some expensive async operation
///     42
/// }
///
/// async fn main() {
///     let result = time_it_async!("expensive_async_operation", expensive_async_operation().await);
///     assert_eq!(result, 42);
/// }
/// ```
#[macro_export]
macro_rules! time_it_async {
    ($name:expr, $block:expr) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::info!("{} completed in {:?}", $name, duration);
        result
    }};
}

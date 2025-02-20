mod enums;
mod repository;

pub use actix_web::{get, web, App, Either, Error, HttpResponse, HttpServer, Responder};
use common::response::Response;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::prelude::*;


// /// 初始化日志系统：设置常规日志和审计日志的输出目标
// pub fn init_logging() {
//     // 创建每日滚动的文件 appender，用于常规日志
//     let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "app.log");
//     let file_layer = fmt::layer()
//         .with_writer(file_appender)
//         .with_span_events(fmt::format::FmtSpan::CLOSE) // 记录 span 关闭时的事件
//         .json()  // 以 JSON 格式输出
//         .with_target(false); // 可根据需要选择是否记录 target 信息
//
//     // 创建每日滚动的文件 appender，用于审计日志
//     let audit_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "audit.log");
//     let audit_layer = fmt::layer()
//         .with_writer(audit_appender)
//         .json()
//         // 仅过滤 target 中包含 "audit" 的事件
//         .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
//             metadata.target().contains("audit")
//         }));
//
//     // 使用上海时区
//     let timer = fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f %z".to_string());
//     // 构建全局 subscriber，并设置为全局默认
//     let subscriber = Registry::default()
//         .with(file_layer)
//         .with(audit_layer);
//
//     tracing::subscriber::set_global_default(subscriber)
//         .expect("无法设置全局 subscriber");
//
// }

// pub fn init_logger() {
//     // 使用上海时区
//     let timer = fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f %z".to_string());
//
//     tracing_subscriber::fmt()
//
//         .json()
//         .with_max_level(tracing::Level::INFO)
//         .with_timer(timer)
//         .with_target(false) // 可选：隐藏目标
//         .with_thread_ids(false) // 可选：隐藏线程ID
//         .with_line_number(true) // 可选：显示行号
//         .init();
// }

type RegisterResult<T> = Either<Response<T>, Result<&'static str, Error>>;

#[get("/index")]
async fn index() -> RegisterResult<&'static str> {
    // if random() == 0 {
    //     // choose Left variant
    //     Either::Left(Response::success("Hello world!"))
    // } else {
    //     // choose Right variant
    //     Either::Right(Ok("Hello!"))
    // }

    Either::Right(Ok("Hello Index!"))
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().body("hello world")
}

#[get("/check/health")]
async fn check_health() -> impl Responder {
    HttpResponse::Ok().body("OK!")
}

#[cfg(test)]
mod tests {
    use crate::{check_health, home, index};
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test] // Actix 测试宏
    async fn test_app() {
        let app =
            test::init_service(
                App::new()
                    .route("/", web::get().to(|| HttpResponse::Ok()))
                    .service(index)
                    .service(home)
                    .service(check_health)
            )
                .await;

        let req = test::TestRequest::get().uri("/check/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        let body = test::read_body(resp).await;
        assert_eq!(body, "OK!");

        println!("test_app 测试完成！！！")
    }

}

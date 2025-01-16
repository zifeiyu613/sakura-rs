pub mod controllers;

use crate::controllers::test_controller::test_controller_config;
use actix_web::middleware::Logger;
pub use actix_web::{get, web, App, Either, Error, HttpResponse, HttpServer, Responder};
use middleware::request_extractor::RequestExtractor;
use sakura_core::response::Response;
use tracing_subscriber::fmt;

type RegisterResult<T> = Either<Response<T>, Result<&'static str, Error>>;

async fn index() -> RegisterResult<&'static str> {
    // if random() == 0 {
    //     // choose Left variant
    //     Either::Left(Response::success("Hello world!"))
    // } else {
    //     // choose Right variant
    //     Either::Right(Ok("Hello!"))
    // }

    Either::Right(Ok("Hello!"))
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().body("hello world")
}


#[actix_web::main]
pub async fn main() {
    // 使用上海时区
    let timer = fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f %z".to_string());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_timer(timer)
        .with_target(false) // 可选：隐藏目标
        .with_thread_ids(false) // 可选：隐藏线程ID
        .with_line_number(true) // 可选：显示行号
        .init();

    // 环境变量初始化

    let addrs = ("127.0.0.1", 8080);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(RequestExtractor::default())
            .service(home)
            .service(web::scope("/test").configure(test_controller_config))
    })
    .bind(addrs)
    .unwrap()
    .run()
    .await
    .unwrap()
}

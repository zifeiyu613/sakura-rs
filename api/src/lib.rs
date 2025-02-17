
pub use actix_web::{get, web, App, Either, Error, HttpResponse, HttpServer, Responder};
use sakura_core::response::Response;
use tracing_subscriber::fmt;


pub fn init_logger() {
    // 使用上海时区
    let timer = fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f %z".to_string());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_timer(timer)
        .with_target(false) // 可选：隐藏目标
        .with_thread_ids(false) // 可选：隐藏线程ID
        .with_line_number(true) // 可选：显示行号
        .init();
}

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

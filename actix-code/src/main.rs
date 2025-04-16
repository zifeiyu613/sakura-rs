use actix_web::{get, guard, post, web, App, Either, Error, HttpResponse, HttpServer, Responder};
use std::ops::Add;
use std::sync::Mutex;
use actix_web::middleware::Logger;

use futures::{StreamExt};
use tracing::info;
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt;

use chrono::{Local, TimeZone, Utc};

pub struct AppState {
    app_name: String,
}

struct AppStateWithCounter {
    counter: Mutex<i32>,
}

#[derive(Serialize, serde::Deserialize)]
struct ReceiptData {

    #[serde(rename = "receipt-data")]
    receipt_data: String,
}


#[post("/json")]
async fn handle_json(mut body: web::Payload) -> Result<HttpResponse, Error> {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        let item = item?;
        println!("Chunk: {:?}", &item);
        bytes.extend_from_slice(&item);
    }

    let bytes = bytes.freeze();
    let obj = serde_json::from_slice::<serde_json::Value>(&bytes)?;
    // HttpResponse::Ok().body(serde_json::to_string_pretty(&obj).unwrap())
    Ok(HttpResponse::Ok().json(obj))
}


// 通用分页数据结构
// #[derive(Serialize)]
// pub struct Paginated<T> {
//     pub list: Vec<T>,
//     pub total: u64,
//     pub page: u32,
//     pub size: u32,
// }

// 通用响应结构
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub status: String,
    pub message: String,
    pub data: Option<T>,
    pub trace_id: Option<String>,
}

impl<T> ApiResponse<T> {
    // 成功返回
    pub fn success(data: T) -> Self {
        ApiResponse {
            code: 200,
            status: "ok".to_string(),
            message: "success".to_string(),
            data: Some(data),
            trace_id: None,
        }
    }

    // 分页成功返回
    // pub fn paginated(data: Paginated<T>) -> Self {
    //     ApiResponse {
    //         code: 200,
    //         status: "ok".to_string(),
    //         message: "success".to_string(),
    //         data: Some(data),
    //         trace_id: None,
    //     }
    // }

    // 错误返回
    pub fn error(code: u16, message: &str, trace_id: Option<String>) -> Self {
        ApiResponse {
            code,
            status: "error".to_string(),
            message: message.to_string(),
            data: None,
            trace_id,
        }
    }
}

// 用户数据结构
#[derive(Serialize)]
struct User {
    id: u32,
    name: String,
    token: String,
}

async fn login_user(user: web::Json<serde_json::Value>) -> Either<web::Json<ApiResponse<User>>, web::Json<ApiResponse<()>>> {

    info!("user: {}", user.to_string());

    // 模拟登录逻辑
    if user.get("username").unwrap() == "admin" && user.get("password").unwrap() == "password" {
        // 登录成功，返回用户信息
        let user = User {
            id: 1,
            name: "Admin".to_string(),
            token: "abcdef123456".to_string(),
        };
        Either::Left(web::Json(ApiResponse::success(user)))
    } else {
        // 登录失败，返回错误信息
        Either::Right(web::Json(ApiResponse::error(401, "Invalid username or password", None)))
    }
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .route(web::get().to(|| async { HttpResponse::Ok().body("app") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 使用上海时区
    let timer = fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f %z".to_string());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_timer(timer)
        .with_target(false)  // 可选：隐藏目标
        .with_thread_ids(false)  // 可选：隐藏线程ID
        .with_line_number(true)  // 可选：显示行号
        .init();

    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });


    let addrs = "127.0.0.1:8088";

    info!("listening on http://{}", &addrs);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            // .wrap(middleware::request_logger::RequestLogger::new(true, true))
            // .wrap(middleware::request_logger_v1::RequestLogger)
            .service(handle_json)
            .configure(config)
            // .configure(rconfig)
            .route("/", web::get().to(index))
            .route(
                "/code",
                web::get().to(|| async { HttpResponse::Ok().body("async code~~~") }),
            )
            .service(web::resource("/").route(web::get().to(index)))
            .service(
                web::scope("/api/v1")
                    .route("/auth", web::post().to(auth))
                    .route("/login", web::post().to(login_user)),
            )
            .app_data(web::Data::new(AppState {
                app_name: "actix-code-ss".to_string(),
            }))
            .service(web::scope("/app").route("/name", web::get().to(app_name)))
            .service(
                web::scope("/counter")
                    .app_data(counter.clone())
                    .route("/", web::get().to(count)),
            )
            .configure(config_scope)
            .service(web::scope("/with-counter").configure(config_scope))
    })
    .bind(addrs)
    .expect("Can not bind to 127.0.0.1:8088")
    .run()
    .await
    .expect("Can not run server");

    Ok(())
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello Actix Code!")
}

async fn auth() -> impl Responder {
    HttpResponse::Ok().body("Hello Actix Authentication Code!")
}

async fn app_name(data: web::Data<AppState>) -> impl Responder {
    let app_name = &data.app_name;
    HttpResponse::Ok().body(format!("Hello Actix App, My Name is {}", app_name))
}

async fn count(data: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter = counter.add(1);
    info!("counter add 1 = {}", counter);
    *counter += 1;
    info!("counter+1 = {}", counter);
    HttpResponse::Ok().body(format!("{:?}", counter))
}

fn config_scope(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/rconfig")
        .route(web::get().to(||async { HttpResponse::Ok().body("Hello Actix Configuration!") })));
}
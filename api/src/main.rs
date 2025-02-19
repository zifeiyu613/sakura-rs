pub mod controllers;
pub mod service;

use std::env;
use crate::controllers::{
    test_controller::test_controller_config, user_controller::user_controller_config,
};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use middleware::RequestExtractor;
use sakura_api::{check_health, home};
use sakura_logs::init_logging;

#[actix_web::main]
pub async fn main() {

    // init_logger();
    init_logging();

    let port = env::var("port").map_or_else(8080);

    let addrs = ("127.0.0.1", 8080);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(RequestExtractor::default())
            // .service(index)
            .service(home)
            .service(check_health)
            .service(web::scope("/test").configure(test_controller_config))
            .service(web::scope("/user").configure(user_controller_config))
    })
    .bind(addrs)
    .unwrap()
    .run()
    .await
    .unwrap()
}

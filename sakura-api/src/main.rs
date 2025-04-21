pub mod controllers;
pub mod service;

use crate::controllers::{
    test_controller::test_controller_config, user_controller::user_controller_config,
};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use clap::{Arg, Command};
use middleware::RequestExtractor;
use rconfig::config::AppConfigBuilder;
use sakura_api::{check_health, home};

#[actix_web::main]
pub async fn main() {
    // 解析命令行参数
    let (port, config_path) = get_command_param();
    // 加载配置文件
    let app_config = AppConfigBuilder::new().add_default(&config_path).build().unwrap();
    // 使用配置初始化日志
    rlog::init_from_config(&app_config).unwrap();

    let addrs = ("127.0.0.1", port);

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

fn get_command_param() -> (u16, String) {
    // 使用 Command 来定义应用程序
    let matches = Command::new("sakura-api")
        .version("1.0")
        .author("will")
        .about("A simple example using Clap 4.x")
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("The port to service"),
        )
        .arg(
            Arg::new("rconfig")
                .short('c')
                .long("rconfig")
                .value_name("CONFIG")
                // .action(clap::ArgAction::SetTrue)
                .help("Config file to use"),
        )
        .get_matches();

    // 获取命令行参数
    let port = matches.get_one::<u16>("port").unwrap_or(&8080);

    let config_path = matches
        .get_one::<String>("config")
        .unwrap_or(&String::from(
            "/Users/will/RustroverProjects/sakura/sakura-api/rconfig.toml",
        ))
        .clone();
    (*port, config_path)
}

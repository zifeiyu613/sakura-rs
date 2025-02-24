use actix_web::{web, HttpResponse, Responder};
use actix_web::web::ServiceConfig;
use sakura_macros::service;
use web_core::web_service::{WebServerManager, WebService};

#[service]
pub struct HomeController;

impl WebService for HomeController {

    fn configure(&self, cfg: &mut ServiceConfig) {
        cfg.service(web::resource("/home").route(web::get().to(Self::home)))
            .service(web::resource("/stop").route(web::get().to(Self::stop)))

        ;
    }
}

impl HomeController {

    async fn home() -> impl Responder {
        HttpResponse::Ok().body("Home controller successfully")
    }

    async fn stop() -> impl Responder {
        let manager = WebServerManager::new(8080);
        manager.stop_server().await;
        HttpResponse::Ok().body("Stopped server successfully")
    }

}


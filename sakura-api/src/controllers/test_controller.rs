use crate::controllers::app_data_extractor::AppData;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use middleware::RequestContext;

pub fn test_controller_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/vd/getBasicInfo")
            .route(web::post().to(get_basic_info))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    )
    .service(web::resource("/vd/postBasicInfo").get(get_basic_info))
    .service(web::resource("/vd/postBasicInfo").post(get_basic_info))
    .service(web::resource("/vd/postBasicInfo").delete(get_basic_info))
    .service(web::resource("/vd/postBasicInfo").to(get_basic_info))
    .service(web::resource("/vd/postBasicInfo").to(get_basic_info))
    ;
}

// #[post("getBasicInfo")]
pub async fn get_basic_info(req: HttpRequest) -> impl Responder {
    if let Some(context) = req.extensions().get::<RequestContext>() {
        println!("RequestContext: {:?}", context);

        let app_data = AppData::new(context);

        if let Some(version) = &app_data.version {
            println!("Application version: {}", version);
        }

        return HttpResponse::Ok().json(&app_data);
    }
    HttpResponse::BadRequest().finish()
}

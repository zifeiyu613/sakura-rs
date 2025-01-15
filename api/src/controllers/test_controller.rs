use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use middleware::RequestContext;
use crate::controllers::app_data_extractor::AppData;

pub fn test_controller_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/vd")
            .route(web::post().to(get_basic_info))
            // .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}


// #[post("getBasicInfo")]
pub async fn get_basic_info(req: HttpRequest) -> impl Responder {

    if let Some(context) =  req.extensions().get::<RequestContext>() {
        println!("RequestContext: {:?}", context);
        let form_data = &context.form_data;
        let data = form_data.data;
        let app_data = AppData::from(context.clone());
        println!("app_data: {:?}", app_data);
        return HttpResponse::Ok().body(serde_json::to_string(&app_data).unwrap())
    }
    HttpResponse::BadRequest().finish()
}
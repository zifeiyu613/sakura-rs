use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use middleware::RequestContext;
use sakura_service::user::user_main_service;
use crate::controllers::app_data_extractor::AppData;

pub fn user_controller_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/get_token").get(get_user_token));
}

pub async fn get_user_token(req: HttpRequest) -> impl Responder {
    if let Some(context) = req.extensions().get::<RequestContext>() {
        println!("uri:{}, RequestContext: {:?}", req.uri(), context);

        let app_data = AppData::new(context);

        if let Some(version) = &app_data.version {
            println!("Application version: {}", version);
        }

        if let Some(uid) = &app_data.uid {
            let token = user_main_service::query_token(uid).await;
            if let Some(token) = token {
                return HttpResponse::Ok().json(token);
            }
            return HttpResponse::Ok().json("Token is not valid");
        }
    }
    HttpResponse::BadRequest().finish()
}

#[cfg(test)]
mod tests {
    use actix_web::{http, test, App};
    use serde_json::json;
    use middleware::RequestExtractor;
    use super::*;

    #[actix_web::test]
    async fn test_get_user_token_valid() {
        let app =
            test::init_service(
                App::new()
                    .wrap(RequestExtractor::default())
                    .route("/user/get_token", web::get().to(get_user_token))
            )
                .await;

        let req = test::TestRequest::get().uri("/user/get_token")
            .insert_header(http::header::ContentType::json())
            .set_json(&json!(
                {"data":{"channel": "Alice", "uid": 2}}
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        let body = test::read_body(resp).await;
        assert_eq!(body, "fb8427e74ac3f7a0d6bb8e58e7a799ad");

        println!("test_get_user_token_valid 测试完成！！！")
    }
}
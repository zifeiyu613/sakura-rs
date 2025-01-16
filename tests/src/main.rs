use actix_web::test;

#[actix_web::test]
async fn test_concurrent_requests() {
    let app = test::init_service(
       sakura_api::main()
    ).await;

    // 模拟多个并发请求
    let futures = (0..100).map(|_| {
        let req = test::TestRequest::get().uri("/").to_request();
        test::call_service(&app, req)
    });

    let results = futures::future::join_all(futures).await;

    for res in results {
        assert!(res.status().is_success());
    }
}
use payment_service::models::payment::{CreatePaymentRequest, RefundRequest};
use payment_service::models::enums::{PaymentType, OrderStatus};
use serde_json::json;
use httpmock::prelude::*;
use reqwest::Client;

#[tokio::test]
#[ignore] // 需要数据库，所以默认忽略
async fn test_payment_flow() -> anyhow::Result<()> {
    // 创建模拟的第三方支付服务器
    let server = MockServer::start();

    // 模拟微信支付统一下单接口
    let wechat_unifiedorder_mock = server.mock(|when, then| {
        when.method("POST")
            .path("/pay/unifiedorder");
        then.status(200)
            .header("Content-Type", "application/json")
            .json_body(json!({
                "return_code": "SUCCESS",
                "result_code": "SUCCESS",
                "prepay_id": "wx123456789",
                "mweb_url": "https://wx.tenpay.com/cgi-bin/mmpayweb-bin/checkmweb?prepay_id=wx123456&package=1234567890"
            }));
    });

    // 模拟微信支付查询接口
    let wechat_query_mock = server.mock(|when, then| {
        when.method("POST")
            .path("/pay/orderquery");
        then.status(200)
            .header("Content-Type", "application/json")
            .json_body(json!({
                "return_code": "SUCCESS",
                "result_code": "SUCCESS",
                "trade_state": "SUCCESS",
                "transaction_id": "4200000123456789",
                "out_trade_no": "{{out_trade_no}}",
                "total_fee": "{{total_fee}}"
            }));
    });

    // 模拟微信支付退款接口
    let wechat_refund_mock = server.mock(|when, then| {
        when.method("POST")
            .path("/pay/refund");
        then.status(200)
            .header("Content-Type", "application/json")
            .json_body(json!({
                "return_code": "SUCCESS",
                "result_code": "SUCCESS",
                "refund_id": "50000123456789",
                "out_refund_no": "{{out_refund_no}}",
                "out_trade_no": "{{out_trade_no}}"
            }));
    });

    // 设置环境变量
    unsafe {
        std::env::set_var("DATABASE_URL", "mysql://root:password@localhost/payment_service_test");
        std::env::set_var("SERVER_PORT", "3001");
    }

    // 启动服务器(在实际测试中，这会使用主函数)
    let server_thread = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let _ = payment_service::main().await;
        });
    });

    // 等待服务启动
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // 创建HTTP客户端
    let client = Client::new();

    // 1. 创建支付订单
    let create_request = CreatePaymentRequest {
        tenant_id: 1,
        user_id: 100,
        payment_type: PaymentType::WxH5,
        amount: 10000,
        currency: "CNY".to_string(),
        product_name: "测试商品".to_string(),
        product_desc: Some("商品描述".to_string()),
        callback_url: Some("http://example.com/callback".to_string()),
        notify_url: Some("http://example.com/notify".to_string()),
        extra_data: None,
    };

    let response = client.post("http://localhost:3001/api/v1/payment/create")
        .json(&create_request)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let response_data: serde_json::Value = response.json().await?;
    assert_eq!(response_data["success"], true);

    let order_id = response_data["data"]["order_id"].as_str().unwrap().to_string();

    // 验证微信支付接口被调用
    wechat_unifiedorder_mock.assert();

    // 2. 查询支付订单
    let response = client.get(&format!("http://localhost:3001/api/v1/payment/query/{}", order_id))
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let response_data: serde_json::Value = response.json().await?;
    assert_eq!(response_data["success"], true);

    // 验证微信查询接口被调用
    wechat_query_mock.assert();

    // 3. 模拟支付回调
    let callback_data = json!({
        "return_code": "SUCCESS",
        "result_code": "SUCCESS",
        "out_trade_no": order_id,
        "transaction_id": "4200000123456789",
        "total_fee": "10000",
        "attach": "测试商品"
    });

    let response = client.post("http://localhost:3001/api/v1/payment/callback/WX_H5?tenant_id=1")
        .json(&callback_data)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    // 4. 再次查询确认状态已更新
    let response = client.get(&format!("http://localhost:3001/api/v1/payment/query/{}", order_id))
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let response_data: serde_json::Value = response.json().await?;
    assert_eq!(response_data["success"], true);
    assert_eq!(response_data["status"], "SUCCESS");

    // 5. 申请退款
    let refund_request = RefundRequest {
        order_id: order_id.clone(),
        refund_amount: 10000,
        refund_reason: Some("测试退款".to_string()),
    };

    let response = client.post("http://localhost:3001/api/v1/payment/refund")
        .json(&refund_request)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let response_data: serde_json::Value = response.json().await?;
    assert_eq!(response_data["success"], true);
    assert!(response_data["refund_id"].is_string());

    // 验证微信退款接口被调用
    wechat_refund_mock.assert();

    // 6. 再次查询确认状态为已退款
    let response = client.get(&format!("http://localhost:3001/api/v1/payment/query/{}", order_id))
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let response_data: serde_json::Value = response.json().await?;
    assert_eq!(response_data["success"], true);
    assert_eq!(response_data["status"], "REFUNDED");

    Ok(())
}
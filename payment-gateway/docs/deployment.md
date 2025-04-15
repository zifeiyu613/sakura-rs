

##

```shell
# 构建项目  
cargo build --release  

# 运行数据库迁移  
cargo run --bin payment-gateway -- migrate  

# 启动服务  
cargo run --release
```

```shell
curl -X POST http://localhost:8080/api/v1/payments \
  -H "Content-Type: application/json" \
  -d '{  
    "merchant_id": "test_merchant",  
    "order_id": "order123456",  
    "amount": "100.00",  
    "currency": "CNY",  
    "subject": "Test Payment",  
    "channel": "wechat",  
    "method": "native",  
    "region": "china",  
    "callback_url": "https://example.com/callback"  
  }'
```

### 查询

```shell
curl -X GET "http://localhost:8080/api/v1/payments?order_id=order123456"
```


### 测试

```shell
# 运行单元测试  
cargo test  

# 运行集成测试  
cargo test --test '*'
```
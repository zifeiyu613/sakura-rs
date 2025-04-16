
## 系统架构图

                                  +-------------------+  
                                  |                   |  
                                  |  客户端应用/商户  |  
                                  |                   |  
                                  +--------+----------+  
                                           |  
                                           | HTTP API  
                                           |  
                  +------------------------v--------------------------+  
                  |                                                   |  
                  |                  支付网关API层                      |  
                  |                                                   |  
                  +-+----------------+-------------------+------------+  
                    |                |                   |  
                    |                |                   |  
            +-------------v----+  +--------v--------+  +------v-------------+  
            |                  |  |                 |  |                    |  
            |   支付服务层     |  |   退款服务层    |  |   通知/回调服务层  |  
            |                  |  |                 |  |                    |  
            +-------------+----+  +--------+--------+  +------+-------------+  
                        |                |                   |  
                        |                |                   |  
            +-------------v----------------v-------------------v-------------+  
            |                                                                |  
            |                       支付处理器工厂                           |  
            |                                                                |  
            +------+---------------+-------------------+--------------------+  
                         |               |                   |  
                         |               |                   |  
            +------------v-----+ +-------v---------+ +-------v-----------+  
            |                  | |                 | |                    |  
            |  微信支付处理器   | |  支付宝处理器   | |  Boost钱包处理器   |  
            |                  | |                 | |                    |  
            +------------------+ +-----------------+ +--------------------+  
            |               |                   |  
            |               |                   |  
            +------------v---------------v-------------------v-------------+  
            |                                                              |  
            |                         数据存储层                            |  
            |            (支付订单、交易记录、退款记录)                      |  
            |                                                              |  
            +--------------------------------------------------------------+


### 项目结构

```angular2html
payment-gateway/  
│  
├── Cargo.toml                    # 项目依赖和配置  
├── Cargo.lock                    # 锁定依赖版本  
├── README.md                     # 项目说明文档  
├── .env                          # 环境变量配置  
├── .gitignore                    # Git忽略文件  
│  
├── migrations/                   # 数据库迁移文件  
│   ├── 20240101000000_init.sql   # 初始化数据库表结构  
│   └── ...  
│  
├── docs/                         # 项目文档  
│   ├── api.md                    # API接口文档  
│   ├── architecture.md           # 架构设计文档  
│   └── deployment.md             # 部署指南  
│  
├── src/                          # 源代码目录  
│   ├── main.rs                   # 程序入口点  
│   ├── lib.rs                    # 库入口点  
│   │  
│   ├── domain/                   # 领域层 - 核心业务逻辑和实体  
│   │   ├── mod.rs                # 领域模块导出  
│   │   ├── models.rs             # 核心领域模型定义  
│   │   ├── payment/              # 支付相关领域逻辑  
│   │   │   ├── mod.rs            # 支付模块导出  
│   │   │   ├── payment.rs        # 支付接口和抽象定义  
│   │   │   └── processor.rs      # 支付处理器接口和工厂  
│   │   │  
│   │   ├── refund/               # 退款相关领域逻辑  
│   │   │   ├── mod.rs            # 退款模块导出  
│   │   │   └── refund.rs         # 退款接口和抽象定义  
│   │   │  
│   │   └── service/              # 领域服务  
│   │       ├── mod.rs            # 服务模块导出  
│   │       ├── payment_service.rs # 支付服务接口  
│   │       └── refund_service.rs  # 退款服务接口  
│   │  
│   ├── application/              # 应用层 - 用例实现  
│   │   ├── mod.rs                # 应用模块导出  
│   │   ├── service/              # 具体服务实现  
│   │   │   ├── mod.rs            # 服务模块导出  
│   │   │   └── payment_service_impl.rs # 支付服务实现  
│   │   │  
│   │   └── dto/                  # 数据传输对象  
│   │       ├── mod.rs            # DTO模块导出  
│   │       ├── payment_dto.rs    # 支付相关DTO  
│   │       └── refund_dto.rs     # 退款相关DTO  
│   │  
│   ├── infrastructure/           # 基础设施层 - 外部依赖实现  
│   │   ├── mod.rs                # 基础设施模块导出  
│   │   ├── config.rs             # 应用配置  
│   │   ├── database.rs           # 数据库连接和初始化  
│   │   │  
│   │   ├── payment/              # 支付渠道实现  
│   │   │   ├── mod.rs            # 支付实现模块导出  
│   │   │   ├── wechat_pay.rs     # 微信支付实现  
│   │   │   ├── alipay.rs         # 支付宝实现  
│   │   │   └── boost_wallet.rs   # Boost钱包实现  
│   │   │  
│   │   ├── repository/           # 数据库仓库实现  
│   │   │   ├── mod.rs            # 仓库模块导出  
│   │   │   ├── payment_order_repository.rs # 支付订单仓库  
│   │   │   ├── payment_transaction_repository.rs # 支付交易仓库  
│   │   │   └── refund_order_repository.rs # 退款订单仓库  
│   │   │  
│   │   └── utils/                # 工具类  
│   │       ├── mod.rs            # 工具模块导出  
│   │       ├── crypto.rs         # 加密工具  
│   │       ├── http_client.rs    # HTTP客户端  
│   │       └── error.rs          # 错误处理  
│   │  
│   ├── interfaces/               # 接口层 - API定义  
│   │   ├── mod.rs                # 接口模块导出  
│   │   ├── api/                  # REST API接口  
│   │   │   ├── mod.rs            # API模块导出  
│   │   │   ├── payment_api.rs    # 支付API  
│   │   │   └── refund_api.rs     # 退款API  
│   │   │  
│   │   ├── dto/                  # 接口数据传输对象  
│   │   │   ├── mod.rs            # DTO模块导出  
│   │   │   ├── request.rs        # 请求数据结构  
│   │   │   └── response.rs       # 响应数据结构  
│   │   │  
│   │   └── middleware/           # Web中间件  
│   │       ├── mod.rs            # 中间件模块导出  
│   │       ├── auth.rs           # 认证中间件  
│   │       └── error_handler.rs  # 错误处理中间件  
│   │  
│   └── server.rs                 # Web服务器配置  
│  
├── tests/                        # 集成测试  
│   ├── api_tests.rs              # API集成测试  
│   ├── payment_tests.rs          # 支付功能测试  
│   └── refund_tests.rs           # 退款功能测试  
│  
└── examples/                     # 示例代码  
├── create_payment.rs         # 创建支付示例  
└── refund_payment.rs         # 退款示例
```

--
请设计一个支付服务系统，需要满足以下需求：

核心接口功能：

订单创建
订单验证
订单查询
退款处理
支付成功通知接收
支持的支付渠道：

微信支付：H5、原生SDK、小程序、扫码支付
支付宝：H5、原生SDK、小程序、扫码支付
云闪付
多种第三方支付服务商
海外支付
新加坡
马来西亚（支持Boost、GrabPay、Touch 'n Go等电子钱包）
其他国家特定支付方式
技术要求：

架构稳定可靠
代码简洁清晰
高可用性设计
易于扩展新支付渠道
提供微服务接口便于业务接入
完善的日志、统计、监控和风控机制
请详细设计此系统并提供完整代码实现。


```shell
payment-service/
├── migrations/               # 数据库迁移文件
├── src/
│   ├── adapters/             # 支付渠道适配器
│   │   ├── alipay/           # 支付宝适配器
│   │   ├── wechat/           # 微信支付适配器
│   │   ├── unionpay/         # 云闪付适配器
│   │   ├── international/    # 国际支付适配器
│   │   └── mod.rs            # 适配器模块导出
│   ├── api/                  # API 层
│   │   ├── handlers/         # 请求处理器
│   │   ├── middleware/       # API中间件
│   │   ├── routes.rs         # API路由定义
│   │   └── mod.rs            # API模块导出
│   ├── config/               # 配置管理
│   │   ├── app_config.rs     # 应用配置
│   │   └── mod.rs            # 配置模块导出
│   ├── domain/               # 领域模型
│   │   ├── entities/         # 实体定义
│   │   ├── enums/            # 枚举定义
│   │   ├── value_objects/    # 值对象
│   │   └── mod.rs            # 领域模块导出
│   ├── infrastructure/       # 基础设施
│   │   ├── cache/            # 缓存访问
│   │   ├── database/         # 数据库访问
│   │   ├── messaging/        # 消息队列
│   │   ├── logging/          # 日志配置
│   │   └── mod.rs            # 基础设施模块导出
│   ├── repositories/         # 仓储层
│   │   ├── order_repo.rs     # 订单仓储
│   │   ├── transaction_repo.rs # 交易仓储
│   │   └── mod.rs            # 仓储模块导出
│   ├── services/             # 服务层
│   │   ├── payment/          # 支付服务
│   │   ├── notification/     # 通知服务
│   │   ├── risk/             # 风控服务
│   │   └── mod.rs            # 服务模块导出
│   ├── utils/                # 工具类
│   │   ├── crypto.rs         # 加密工具
│   │   ├── errors.rs         # 错误定义
│   │   ├── validator.rs      # 数据验证
│   │   └── mod.rs            # 工具模块导出
│   ├── app_state.rs          # 应用状态
│   └── main.rs               # 应用入口
├── .env                      # 环境变量
├── Cargo.toml                # 项目依赖
└── README.md                 # 项目说明

```
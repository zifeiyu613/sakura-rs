# sakura

Personal Pro

## Getting started

To make it easy for you to get started with GitLab, here's a list of recommended next steps.

Already a pro? Just edit this README.md and make it your own. Want to make it easy? [Use the template at the bottom](#editing-this-readme)!

## Add your files

- [ ] [Create](https://docs.gitlab.com/ee/user/project/repository/web_editor.html#create-a-file) or [upload](https://docs.gitlab.com/ee/user/project/repository/web_editor.html#upload-a-file) files
- [ ] [Add files using the command line](https://docs.gitlab.com/ee/gitlab-basics/add-file.html#add-a-file-using-the-command-line) or push an existing Git repository with the following command:

```
cd existing_repo
git remote add origin https://gitlab.kaiqi.xin/qiangren/sakura.git
git branch -M main
git push -uf origin main
```

## Sakura Web Service

A Rust web service for managing multiple microservices like database, message queues, etc.


## 项目结构
```
sakura/                      # 🌲 根目录 (workspace)
│── Cargo.toml               # 🏠 Cargo workspace
│── sakura-web/              # 🌐 Web 服务
│   ├── src/
│   │   ├── api/             # 控制器 (路由)
│   │   ├── service/         # 业务逻辑层
│   │   ├── repository/      # 数据库交互层
│   │   ├── app.rs           # 应用入口
│   │   ├── main.rs          # 🌟 服务器启动
│   ├── Cargo.toml
│── sakura-cli/              # 🛠 CLI 工具 (管理 Web 服务)
│   ├── src/
│   │   ├── commands.rs      # CLI 命令 (start, stop, status)
│   │   ├── main.rs
│   ├── Cargo.toml
│── crates/                  # 📦 公共库
│   ├── common/              # 通用工具 (日志, 配置, 错误)
│   ├── config/              # 配置解析 (环境变量, TOML, YAML)
│   ├── errors/              # 统一错误处理
│   ├── mq/                  # MQ 组件 (RabbitMQ)
│   ├── database/            # 数据库组件 (MySQL, PostgreSQL)
│   ├── redis/               # Redis 组件
│   ├── web-core/            # Web 框架核心 (提供 WebService trait)
│── .gitignore
│── README.md

```
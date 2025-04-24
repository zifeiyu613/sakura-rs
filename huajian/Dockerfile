# 开发阶段基础镜像
FROM rust:1.73 as builder

WORKDIR /app

# huajian 是工作区中的一个子模块
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

# 如果存在工作区配置，也复制它
COPY ./Cargo.workspace.toml ./Cargo.workspace.toml

# 只复制子模块 api-service
COPY ./huajian ./huajian

# 构建子模块
WORKDIR /app/huajian
RUN cargo build --release

# 运行镜像
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/huajian /app/huajian
COPY --from=builder /app/huajian/.env.example /app/.env

# 设置环境变量
ENV RUST_LOG=info

# 暴露 API 端口
EXPOSE 8080

# 添加健康检查
HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# 运行应用
CMD ["./huajian"]

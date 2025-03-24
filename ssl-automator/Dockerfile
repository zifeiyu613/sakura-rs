# 构建阶段
FROM rust:1.85-slim as builder

WORKDIR /app

# 安装依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制项目文件
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# 构建项目
RUN cargo build --release

# 运行阶段 - 使用更小的基础镜像
FROM debian:bullseye-slim

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# 复制构建好的可执行文件
COPY --from=builder /app/target/release/ssl-automator /usr/local/bin/

# 添加启动脚本
COPY scripts/entrypoint.sh /
RUN chmod +x /entrypoint.sh

# 创建工作目录
WORKDIR /app

# 创建必要的目录
RUN mkdir -p /config /certs /webroots

# 设置环境变量
ENV RUST_LOG=info
ENV CONFIG_PATH=/config/config.json
ENV DB_PATH=/config/certs.db
ENV ACME_DIR=/config/acme

# 暴露健康检查端口(如果应用有HTTP端口)
# EXPOSE 8080

# 定义卷
VOLUME ["/config", "/certs", "/webroots"]

# 设置入口点
ENTRYPOINT ["/entrypoint.sh"]

# 默认命令
CMD ["serve"]

# 添加标签
LABEL maintainer="Your Name <your.email@example.com>"
LABEL version="1.0.0"
LABEL description="Automated SSL certificate management with Let's Encrypt"

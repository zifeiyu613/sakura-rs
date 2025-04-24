# 构建阶段
FROM rust:1.73 as builder

# 设置工作目录为根工作区
WORKDIR /workspace

# 首先复制整个工作区的 Cargo 配置
COPY ../Cargo.toml ../Cargo.lock ./

# 创建临时目录结构以满足 Cargo 工作区的需求
# 这一步骤避免了复制所有子项目的源代码，只复制 huajian
RUN mkdir -p huajian other_projects

# 复制 huajian 项目的 Cargo.toml 进行依赖预构建
COPY ./Cargo.toml ./huajian/

# 如果有其他子项目，可以创建它们的空目录结构
# 这里我们为 other_projects 创建一个最小化的 Cargo.toml 以满足工作区需求
RUN echo '[package]
name = "other_projects"
version = "0.1.0"
edition = "2021"' > other_projects/Cargo.toml

# 尝试预构建依赖项（这将缓存依赖，当源代码改变时可以重用）
# 创建一个空的 lib.rs 文件，让 Cargo 能够编译依赖
RUN mkdir -p huajian/src && \
    echo "// Empty file for dependency building" > huajian/src/lib.rs && \
    mkdir -p other_projects/src && \
    echo "// Empty file" > other_projects/src/lib.rs && \
    cargo build --release --package huajian

# 现在删除临时文件
RUN rm -rf huajian/src other_projects/src

# 复制实际的源代码
COPY ./src ./huajian/src

# 构建应用程序
RUN cargo build --release --package huajian

# 运行阶段
FROM debian:bullseye-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /workspace/target/release/huajian /app/huajian

# 复制配置文件
COPY ./config /app/config
COPY ./.env.example /app/.env

# 设置环境变量
ENV RUST_LOG=info

# 暴露 API 端口
EXPOSE 8080

# 添加健康检查
HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# 指定启动命令
CMD ["./huajian"]



## 使用 GitHub Actions 自动构建和发布
创建 .github/workflows/docker-publish.yml 文件：

```yaml
name: Docker

on:
  push:
    branches: [ main ]
    tags: [ 'v*.*.*' ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout repository
        uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Login to DockerHub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v4
        with:
          images: yourname/ssl-automator
          tags: |
            type=semver,pattern={{version}}
            type=ref,event=branch
            type=sha

      - name: Build and push
        uses: docker/build-push-action@v3
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

```

## 服务器部署指南

### 快速启动(单个容器)

```shell
# 创建必要的目录
mkdir -p ~/ssl-automator/{config,certs,webroots}

# 运行容器
docker run -d --name ssl-automator \
  -v ~/ssl-automator/config:/config \
  -v ~/ssl-automator/certs:/certs \
  -v ~/ssl-automator/webroots:/webroots \
  -e RUST_LOG=info \
  --restart unless-stopped \
  yourname/ssl-automator:latest

```

### 使用 docker-compose 部署

```shell
# 创建项目目录
mkdir -p ~/ssl-automator
cd ~/ssl-automator

# 下载docker-compose.yml
curl -O https://raw.githubusercontent.com/yourusername/ssl-automator/main/docker-compose.yml

# 创建webroots目录和nginx配置目录
mkdir -p webroots nginx/conf.d

# 启动服务
docker-compose up -d

```

### 添加域名

```shell
# 使用运行中的容器添加域名
docker exec ssl-automator add-domain \
  --domain example.com \
  --webroot /webroots/example.com \
  --cert-dir /certs/example.com \
  --restart-cmd "docker exec nginx nginx -s reload"

# 或使用临时容器
docker run --rm \
  -v ~/ssl-automator/config:/config \
  -v ~/ssl-automator/certs:/certs \
  -v ~/ssl-automator/webroots:/webroots \
  yourname/ssl-automator:latest add-domain \
  --domain example.com \
  --webroot /webroots/example.com \
  --cert-dir /certs/example.com \
  --restart-cmd "docker exec nginx nginx -s reload"

```

### 立即检查所有证书

```shell
docker exec ssl-automator check-now

```
### Nginx 配置示例

在 ~/ssl-automator/nginx/conf.d/example.com.conf 创建配置文件：

```shell
server {
    listen 80;
    server_name example.com www.example.com;

    # ACME 验证目录
    location /.well-known/acme-challenge/ {
        root /var/www/example.com;
    }

    # 重定向到HTTPS
    location / {
        return 301 https://$host$request_uri;
    }
}

server {
    listen 443 ssl http2;
    server_name example.com www.example.com;

    # SSL 配置
    ssl_certificate /etc/ssl/certs/example.com/fullchain.pem;
    ssl_certificate_key /etc/ssl/certs/example.com/privkey.pem;
    
    # 推荐的SSL设置
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:10m;
    ssl_session_tickets off;
    
    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    
    # 网站根目录
    root /var/www/example.com;
    index index.html;

    location / {
        try_files $uri $uri/ =404;
    }
}

```

### 自动更新镜像
创建 ~/ssl-automator/update.sh 脚本：

```shell
#!/bin/bash
# 自动更新ssl-automator容器

cd ~/ssl-automator
docker-compose pull
docker-compose up -d

```

设置为cron任务：
```shell
# 编辑crontab
crontab -e

# 添加每周更新任务
0 3 * * 0 ~/ssl-automator/update.sh >> ~/ssl-automator/update.log 2>&1

```

### 生产环境注意事项

编辑 ~/ssl-automator/config/config.json：
```shell
{
  "email": "your-real-email@example.com",
  "production": true,
  "renew_before_days": 30,
  "domains": [],
  "db_path": "/config/certs.db",
  "acme_dir": "/config/acme"
}

```

### 启用证书通知

如果您的应用支持，配置邮件通知：

```shell
{
  "email": "your-real-email@example.com",
  "production": true,
  "renew_before_days": 30,
  "notification": {
    "smtp_server": "smtp.example.com",
    "smtp_port": 587,
    "username": "notify@example.com",
    "password": "your-password",
    "recipients": ["admin@example.com"]
  },
  "domains": [],
  "db_path": "/config/certs.db",
  "acme_dir": "/config/acme"
}

```

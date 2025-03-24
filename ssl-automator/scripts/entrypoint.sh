#!/bin/bash
set -e

# 确保必要的目录存在
mkdir -p /config /certs /webroots
mkdir -p /config/acme

# 设置正确的权限
chmod 700 /config/acme

# 创建默认配置文件(如果不存在)
if [ ! -f "$CONFIG_PATH" ]; then
  echo "Creating default config at $CONFIG_PATH..."
  cat > "$CONFIG_PATH" <<EOL
{
  "email": "admin@example.com",
  "production": false,
  "renew_before_days": 30,
  "domains": [],
  "db_path": "$DB_PATH",
  "acme_dir": "$ACME_DIR"
}
EOL
fi

# 根据命令参数执行不同的操作
case "\$1" in
  serve)
    echo "Starting SSL Automator in service mode..."
    exec ssl-automator --config "$CONFIG_PATH"
    ;;
  check-now)
    echo "Running immediate certificate check..."
    exec ssl-automator --config "$CONFIG_PATH" check-now
    ;;
  add-domain)
    echo "Adding domain configuration..."
    shift
    exec ssl-automator --config "$CONFIG_PATH" add-domain "$@"
    ;;
  shell)
    echo "Starting shell..."
    exec /bin/bash
    ;;
  *)
    echo "Running custom command: $@"
    exec "$@"
    ;;
esac

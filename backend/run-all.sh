#!/bin/bash

set -euo pipefail

echo "启动后端服务 (单进程)..."

if [ -z "${DATABASE_URL:-}" ]; then
  echo "请先设置 DATABASE_URL 环境变量"
  exit 1
fi

if [ -z "${REDIS_URL:-}" ]; then
  echo "请先设置 REDIS_URL 环境变量"
  exit 1
fi

if [ -z "${AI_SERVICE_URL:-}" ]; then
  echo "请先设置 AI_SERVICE_URL 环境变量"
  exit 1
fi

cargo run

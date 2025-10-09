#!/bin/bash

# 果蔬智造核心平台一键启动脚本
set -euo pipefail

GREEN="\\033[32m"
RESET="\\033[0m"

banner() {
  echo "========================================="
  echo "果蔬智造核心平台 - Docker 部署"
  echo "========================================="
}

check_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "命令 $1 未找到，请先安装后再运行此脚本。"
    exit 1
  fi
}

banner

check_command docker
check_command "docker compose"

if [ ! -f .env ]; then
  echo "未发现 .env，正在从 .env.example 创建..."
  cp .env.example .env
  echo -e "${GREEN}✓ .env 已创建，可根据需要调整配置${RESET}"
fi

echo "停止旧容器..."
docker compose down --remove-orphans >/dev/null 2>&1 || true

services=(postgres redis ai backend frontend)

for svc in "${services[@]}"; do
  echo "构建镜像: $svc"
  docker compose build "$svc"
done

echo "启动所有服务..."
docker compose up -d

wait_for_health() {
  local name=$1
  local retries=30
  local delay=2

  echo "等待服务 ${name} 就绪..."
  for ((i=1; i<=retries; i++)); do
    status=$(docker compose ps --status=running --services | grep "^${name}$" || true)
    if [ -n "$status" ]; then
      echo -e "${GREEN}✓ ${name} 已启动${RESET}"
      return 0
    fi
    sleep "$delay"
  done
  echo "${name} 启动超时，请检查日志。"
  docker compose logs "$name"
  exit 1
}

wait_for_health postgres
wait_for_health redis
wait_for_health ai
wait_for_health backend
wait_for_health frontend

echo "部署完成!"
echo
printf "%-15s %s\n" "前端界面" "http://localhost:3000"
printf "%-15s %s\n" "后端 API" "http://localhost:8080/api/v1"
printf "%-15s %s\n" "AI 服务" "http://localhost:8002/health"
printf "%-15s %s\n" "PostgreSQL" "postgres://gyz:gyz123@localhost:5432/gyz"
printf "%-15s %s\n" "Redis" "redis://localhost:6379"
echo
echo "常用命令:"
printf "  %-30s %s\n" "查看服务状态" "docker compose ps"
printf "  %-30s %s\n" "查看日志" "docker compose logs -f [service]"
printf "  %-30s %s\n" "停止所有服务" "docker compose down"
echo

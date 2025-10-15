#!/bin/bash

set -euo pipefail

banner() {
  echo "========================================="
  echo "果蔬智造核心平台 - 开发环境助手"
  echo "========================================="
}

check_dependencies() {
  echo "检查本地依赖..."

  if command -v cargo >/dev/null 2>&1; then
    echo "✓ Rust: $(cargo --version)"
  else
    echo "⚠ 未检测到 Rust，请按照 https://www.rust-lang.org 安装"
  fi

  if command -v node >/dev/null 2>&1; then
    echo "✓ Node.js: $(node --version)"
  else
    echo "⚠ 未检测到 Node.js，请访问 https://nodejs.org 安装"
  fi

  if command -v python3 >/dev/null 2>&1; then
    echo "✓ Python: $(python3 --version)"
  else
    echo "⚠ 未检测到 Python 3，请安装 Python 3.10+"
  fi

  echo
}

start_infra() {
  if ! command -v docker >/dev/null 2>&1; then
    echo "⚠ 未检测到 Docker，跳过基础设施启动。"
    return
  fi

  echo "启动 PostgreSQL 与 Redis..."
  docker compose up -d postgres redis
  echo "查看状态: docker compose ps"
  echo
}

show_guides() {
  cat <<'GUIDE'
后端 (Rust Axum)
  cd backend
  export DATABASE_URL=postgres://gyz:gyz123@localhost:5432/gyz
  export REDIS_URL=redis://localhost:6379
  export AI_SERVICE_URL=http://localhost:8002
  cargo run

AI 服务 (FastAPI)
  cd ai
  python -m venv .venv && source .venv/bin/activate  # 可选
  pip install -r requirements.txt
  uvicorn main:app --reload --host 0.0.0.0 --port 8002

前端 (Vue 3 + Vite)
  cd frontend
  npm install
  npm run dev

GUIDE
}

main() {
  banner
  check_dependencies

  read -p "是否启动 Docker 基础设施 (Postgres/Redis)? [Y/n] " reply
  if [[ -z "$reply" || "$reply" =~ ^[Yy]$ ]]; then
    start_infra
  else
    echo "跳过 Docker 基础设施启动。"
    echo
  fi

  show_guides

  echo "提示: 若需完整容器化体验，可执行 ./scripts/start.sh"
}

main "$@"

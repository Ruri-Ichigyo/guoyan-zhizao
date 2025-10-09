#!/bin/bash

set -euo pipefail

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

passed=0
failed=0
warnings=0

pass() { echo -e "${GREEN}✓${NC} $1"; passed=$((passed+1)); }
fail() { echo -e "${RED}✗${NC} $1"; failed=$((failed+1)); }
warn() { echo -e "${YELLOW}⚠${NC} $1"; warnings=$((warnings+1)); }

section() {
  echo
  echo "========================================="
  echo "$1"
  echo "========================================="
}

section "核心文件检查"

[ -f docker-compose.yml ] && pass "docker-compose.yml 存在" || fail "缺少 docker-compose.yml"
[ -f .env.example ] && pass ".env.example 存在" || fail "缺少 .env.example"
[ -x scripts/start.sh ] && pass "scripts/start.sh 可执行" || fail "scripts/start.sh 缺失或不可执行"
[ -x scripts/dev-start.sh ] && pass "scripts/dev-start.sh 可执行" || fail "scripts/dev-start.sh 缺失或不可执行"

section "后端 (backend)"

if [ -d backend ]; then
  pass "backend 目录存在"
  [ -f backend/Cargo.toml ] && pass "Cargo.toml 存在" || fail "缺少 backend/Cargo.toml"
  [ -f backend/src/main.rs ] && pass "src/main.rs 存在" || fail "缺少 backend/src/main.rs"
else
  fail "backend 目录不存在"
fi

section "AI 服务 (ai)"

if [ -d ai ]; then
  pass "ai 目录存在"
  [ -f ai/main.py ] && pass "main.py 存在" || fail "缺少 ai/main.py"
  [ -f ai/requirements.txt ] && pass "requirements.txt 存在" || fail "缺少 ai/requirements.txt"
  [ -f ai/Dockerfile ] && pass "Dockerfile 存在" || fail "缺少 ai/Dockerfile"
  if [ -d ai/model ]; then
    pass "模型目录 ai/model 存在"
  else
    warn "未找到 ai/model，推理将使用随机权重"
  fi
else
  fail "ai 目录不存在"
fi

section "前端 (frontend)"

if [ -d frontend ]; then
  pass "frontend 目录存在"
  [ -f frontend/package.json ] && pass "package.json 存在" || fail "缺少 frontend/package.json"
  [ -f frontend/vite.config.ts ] && pass "vite.config.ts 存在" || fail "缺少 frontend/vite.config.ts"
  [ -f frontend/Dockerfile ] && pass "Dockerfile 存在" || fail "缺少 frontend/Dockerfile"
else
  fail "frontend 目录不存在"
fi

section "依赖环境"

if command -v docker >/dev/null 2>&1; then
  pass "Docker 已安装 ($(docker --version | cut -d',' -f1))"
  if docker compose version >/dev/null 2>&1; then
    pass "Docker Compose 已安装"
  else
    warn "Docker Compose 未安装"
  fi
else
  warn "Docker 未安装"
fi

command -v cargo >/dev/null 2>&1 && pass "Rust 工具链已安装" || warn "Rust 未安装"
command -v node >/dev/null 2>&1 && pass "Node.js 已安装" || warn "Node.js 未安装"
command -v python3 >/dev/null 2>&1 && pass "Python3 已安装" || warn "Python3 未安装"

section "验证总结"

total=$((passed + failed + warnings))
cat <<SUMMARY
检查项: $total
通过: $passed
失败: $failed
警告: $warnings
SUMMARY

echo
if [ $failed -eq 0 ]; then
  echo -e "${GREEN}✓ 核心结构已就绪，可执行 ./scripts/start.sh 体验一体化部署${NC}"
else
  echo -e "${RED}✗ 存在阻塞项，请先修复失败检查${NC}"
  exit 1
fi

echo
if [ $warnings -gt 0 ]; then
  echo -e "${YELLOW}注意: 有 $warnings 个警告项，可根据需要处理${NC}"
fi

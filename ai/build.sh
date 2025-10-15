#!/bin/bash
# build.sh

# 启用BuildKit
export DOCKER_BUILDKIT=1

# 使用缓存挂载构建镜像
docker build \
  --cache-from=fruit-detection:latest \
  --cache-to=type=inline \
  -t fruit-detection:latest .
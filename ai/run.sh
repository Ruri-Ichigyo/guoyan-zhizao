#!/bin/bash
# run.sh

# 创建本地缓存目录
mkdir -p ~/.cache/uv

# 运行容器并挂载uv缓存目录
docker run -d \
  --name fruit-detection \
  -p 8002:8002 \
  -v ~/.cache/uv:/root/.cache/uv \
  -v $(pwd)/model:/app/model \
  fruit-detection:latest
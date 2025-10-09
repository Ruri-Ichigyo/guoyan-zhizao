# 果蔬智能分拣系统

## 系统架构

一个基于微服务架构的果蔬智能分拣系统，支持图像上传、AI识别、实时数据展示和分拣控制。

### 技术栈
- **网关**: Kong (JWT认证 + 限流)
- **后端**: Rust微服务 (Tokio + Axum)
- **存储**: MinIO对象存储
- **消息队列**: Apache Kafka
- **AI推理**: Python + DenseNet121
- **前端**: Vue3 + TypeScript + Vite
- **监控**: WebSocket实时通信

## 快速启动

```bash
# 克隆项目
git clone git@github.com:lanshi17/guoyan-zhizao.git
cd guoyan-zhizao

# 启动所有服务 (Docker Compose)
docker-compose up -d

# 访问前端
http://localhost:3000

# API网关
http://localhost:8000

# 统一 API 前缀
http://localhost:8000/api/v1
```

## 服务端口

| 服务 | 端口 | 描述 |
|------|------|------|
| Kong Gateway | 8000 | API网关 |
| Upload Service | 3001 | 文件上传服务 |
| Query Service | 3002 | 查询与WebSocket |
| Sort Service | 3003 | 分拣控制服务 |
| AI Inference | 3004 | AI推理服务 |
| Frontend | 3000 | 前端界面 |
| MinIO | 9000 | 对象存储 |
| MinIO Console | 9001 | 存储管理界面 |

## 开发指南

### 后端开发
```bash
cd backend
cargo build --release
```

### 前端开发
```bash
cd frontend
npm install
npm run dev
```

### AI服务开发
```bash
cd ai
pip install -r requirements.txt
python main.py
```

## 部署说明

详见 [DEPLOY.md](docs/DEPLOY.md) 生产环境部署指南。

## API文档

详见 [API.md](docs/API.md) 完整接口文档。

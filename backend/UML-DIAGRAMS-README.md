# 果蔬检测分选系统 - UML 图表文档

本文档包含了果蔬检测分选系统后端的完整 UML 图表，帮助理解系统架构、数据流和业务流程。

## 📋 图表概览

### 🏗️ 系统架构类图表

| 图表文件 | 描述 | 用途 |
|---------|------|------|
| `architecture-diagram.puml` | 系统整体架构图 | 展示所有服务组件、中间件和数据存储的关系 |
| `sequence-diagram.puml` | 业务流程序列图 | 详细展示各服务间的交互时序 |
| `workflow-diagram.puml` | 工作流程图 | 业务流程的活动图表示 |
| `dataflow-diagram.puml` | 数据流程图 | 数据在系统中的流转过程 |

## 🎯 系统架构概述

### 核心服务组件

1. **Upload Service (上传服务)** - 端口 3001
   - 处理文件上传（直接上传 + 预签名URL）
   - 集成 MinIO 对象存储
   - 发送 Kafka 事件通知

2. **Query Service (查询服务)** - 端口 3002
   - 提供检测结果查询 API
   - WebSocket 实时数据推送
   - 支持分页和状态筛选

3. **Sort Service (分选服务)** - 端口 3003
   - 分选设备控制
   - 状态监控和配置管理
   - 设备命令处理

### 外部依赖组件

- **MinIO 对象存储**: S3兼容的文件存储服务
- **Apache Kafka**: 消息队列，实现异步通信
- **PostgreSQL**: 关系数据库，存储检测结果
- **Redis**: 缓存系统（可选）
- **AI检测服务**: 外部AI模型服务

## 🔄 核心业务流程

### 1. 文件上传流程
```
用户上传 → Upload Service → MinIO存储 → Kafka事件 → AI处理
```

### 2. 检测处理流程  
```
AI服务监听 → 下载图片 → 执行检测 → 保存结果 → 通知完成
```

### 3. 结果查询流程
```
用户查询 → Query Service → 数据库查询 → 返回结果
```

### 4. 实时推送流程
```
检测完成 → Kafka事件 → Query Service → WebSocket广播 → 前端更新
```

### 5. 分选控制流程
```
分选命令 → Sort Service → 设备控制器 → 执行分选 → 状态反馈
```

## 📊 数据结构设计

### 核心数据模型

#### UploadResponse
```json
{
  "upload_id": "string",
  "presigned_url": "string?",
  "status": "uploaded|presigned"
}
```

#### DetectionResult  
```json
{
  "class_name": "string",
  "confidence": "float",
  "bbox": "[x1,y1,x2,y2]",
  "quality_grade": "A|B|C"
}
```

#### ResultResponse
```json
{
  "job_id": "string",
  "status": "string", 
  "results": "DetectionResult[]",
  "created_at": "ISO8601"
}
```

#### SortCommand
```json
{
  "job_id": "string",
  "action": "start|stop|pause|resume",
  "target_grade": "A|B|C?",
  "conveyor_speed": "number?"
}
```

## 🔌 API 接口设计

### Upload Service API
- `POST /api/v1/uploads` - 直接文件上传
- `POST /api/v1/uploads/presign` - 生成预签名URL
- `GET /health` - 健康检查

### Query Service API  
- `GET /api/v1/results/{job_id}` - 获取检测结果
- `GET /api/v1/results` - 分页查询结果列表
- `GET /ws/live` - WebSocket实时推送
- `GET /health` - 健康检查

### Sort Service API
- `POST /api/v1/sort/command` - 发送分选命令  
- `GET /api/v1/sort/status` - 获取分选状态
- `PUT /api/v1/sort/config` - 更新分选配置
- `GET /health` - 健康检查

## 🔄 事件驱动架构

### Kafka 主题设计

| 主题名称 | 描述 | 生产者 | 消费者 |
|---------|------|-------|-------|
| `image-events` | 图片存储事件 | Upload Service | AI检测服务 |
| `detection-completed` | 检测完成事件 | AI检测服务 | Query Service |
| `system-notifications` | 系统通知事件 | 各服务 | 监控系统 |

### 事件格式

#### image.stored 事件
```json
{
  "event_type": "image.stored",
  "upload_id": "uuid", 
  "object_key": "uploads/{id}/{filename}",
  "timestamp": "ISO8601"
}
```

#### detection.completed 事件
```json
{
  "event_type": "detection.completed",
  "job_id": "uuid",
  "results": "DetectionResult[]",
  "statistics": "object"
}
```

## 🛠️ 技术栈说明

### 后端技术
- **Rust**: 系统编程语言，高性能和内存安全
- **Axum**: 现代化的Web框架，支持异步和类型安全
- **Tokio**: 异步运行时，处理并发任务
- **Tower**: 中间件生态，提供CORS、日志等功能

### 中间件技术
- **MinIO**: S3兼容的对象存储，支持分布式部署
- **Apache Kafka**: 高吞吐量消息队列，事件驱动架构核心
- **PostgreSQL**: 关系数据库，ACID事务保证
- **Redis**: 高性能缓存，提升查询速度

### 通信协议
- **HTTP/HTTPS**: REST API 通信
- **WebSocket**: 双向实时通信
- **AWS S3 API**: 对象存储访问协议

## 📈 扩展性设计

### 水平扩展
- 各服务无状态设计，支持多实例部署
- Kafka分区机制，支持消息并行处理
- MinIO分布式存储，支持容量扩展
- 数据库读写分离，支持查询扩展

### 高可用性
- 服务健康检查接口
- Kafka消息持久化和重试机制
- 数据库主从复制
- 负载均衡和故障转移

## 🔧 部署架构

### 容器化部署
```yaml
services:
  upload-svc:
    image: upload-service:latest
    ports: ["3001:3001"]
    
  query-svc: 
    image: query-service:latest
    ports: ["3002:3002"]
    
  sort-svc:
    image: sort-service:latest  
    ports: ["3003:3003"]
    
  minio:
    image: minio/minio:latest
    ports: ["9000:9000"]
    
  kafka:
    image: confluentinc/cp-kafka:latest
    ports: ["9092:9092"]
    
  postgres:
    image: postgres:15
    ports: ["5432:5432"]
```

### 环境配置
- 开发环境：单机Docker Compose
- 测试环境：Kubernetes集群
- 生产环境：云原生部署 + 监控告警

## 📝 使用说明

### 生成图表
1. 安装 PlantUML
2. 使用以下命令生成PNG图片：
```bash
plantuml sequence-diagram.puml
plantuml architecture-diagram.puml  
plantuml workflow-diagram.puml
plantuml dataflow-diagram.puml
```

### 查看图表
生成的PNG文件可以直接在浏览器或图片查看器中打开，也可以嵌入到文档中。

---

**最后更新**: 2025年10月2日  
**版本**: v1.0  
**维护者**: 果蔬智造项目组

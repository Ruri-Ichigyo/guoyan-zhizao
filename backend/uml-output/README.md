# 果蔬智造项目 UML 图形文档

本文档包含了整个果蔬智造 backend 项目的 UML 图形，帮助理解项目架构和代码结构。

## 📁 生成的 UML 图形文件列表

### 🏗️ 项目整体架构图

- **果蔬检测系统-架构图.png** - 完整的系统架构图，展示所有服务、中间件和数据存储的关系
- **果蔬检测系统-序列图.png** - 详细的时序图，展示从文件上传到结果获取的完整交互流程
- **果蔬检测系统-工作流图.png** - 业务流程图，展示用户操作和系统处理的工作流程
- **果蔬检测系统-数据流图.png** - 数据流向图，展示数据在各个组件间的流动和转换

### 📋 原有 UML 图表

- **project-architecture.png** - 展示整个系统的高层架构，包括三个主要服务及其相互关系
- **detailed-classes.png** - 详细的类图，显示各服务中的具体数据结构和方法
- **sequence-flow.png** - 序列图，展示用户上传图片到获取结果的完整流程

### 🔍 各服务模块 UML 图

- **main.png** / **main..png** - 主要服务入口点的结构图
- **routes..png** - API 路由处理器的结构图，包含查询相关的数据结构
- **ws..png** - WebSocket 处理模块的结构图
- **kafka..png** - Kafka 消息队列处理模块的结构图
- **minio..png** - MinIO 对象存储客户端模块的结构图

## 📊 最新图表说明

### 1. 系统架构图 (果蔬检测系统-架构图.png)

展示了完整的微服务架构，包括：

- **用户界面层**: Web 前端、移动端 App
- **API 网关层**: 负载均衡器、API 网关
- **微服务层**: Upload Service、Query Service、Sort Service
- **外部服务层**: AI 检测服务
- **中间件层**: Kafka 消息队列、MinIO 对象存储
- **数据存储层**: PostgreSQL 数据库、Redis 缓存
- **硬件设备层**: 分选设备控制器、传送带系统
- **监控运维层**: 日志收集、性能监控、告警系统

### 2. 序列图 (果蔬检测系统-序列图.png)

详细展示了系统的完整交互时序：

- **文件上传阶段**: 直接上传和预签名 URL 上传两种方式
- **实时通知订阅**: WebSocket 连接建立和心跳检测
- **AI 处理阶段**: 从 Kafka 消费事件到 AI 检测完成
- **结果查询阶段**: 单个结果查询和列表分页查询
- **实时推送阶段**: 检测完成后的实时通知
- **分选控制阶段**: 分选命令和状态监控
- **健康检查**: 各服务的健康状态检查

### 3. 工作流图 (果蔬检测系统-工作流图.png)

展示了完整的业务处理流程：

- **用户操作流程**: 从图片选择到结果查看
- **系统处理流程**: 上传、检测、分选的完整链路
- **分支决策**: 不同上传方式、查询方式、分选需求的处理
- **配置管理**: 系统参数配置和调整
- **技术栈说明**: 使用的核心技术和部署架构

### 4. 数据流图 (果蔬检测系统-数据流图.png)

展示了数据的完整流向：

- **数据输入层**: 原始图片文件格式和要求
- **数据传输层**: 上传方式和存储结构
- **事件驱动层**: Kafka 消息格式和事件类型
- **AI 处理层**: 图像预处理、模型推理、结果整合
- **数据存储层**: 数据库表结构和缓存策略
- **数据分发层**: 实时推送和 API 查询
- **数据查询层**: REST API 响应格式
- **分选控制层**: 命令和状态数据结构

## 🎯 系统架构说明

### 三大核心服务：

1. **Upload Service (上传服务)** - 端口 3001

   - 负责接收用户上传的图片
   - 将文件存储到 MinIO 对象存储
   - 发送处理任务到 Kafka 消息队列
   - 支持直接上传和预签名 URL 两种方式

2. **Query Service (查询服务)** - 端口 3002

   - 提供结果查询 API (REST + 分页)
   - 支持状态筛选和历史查询
   - 实现 WebSocket 实时推送功能
   - 管理检测结果缓存

3. **Sort Service (分选服务)** - 端口 3003
   - 处理分选设备控制命令
   - 实时监控设备状态
   - 配置管理和参数调整
   - 分选统计和报告生成

### 🔄 完整数据流程：

1. **上传阶段**: 用户 → 前端 →Upload Service→MinIO→Kafka 事件
2. **处理阶段**: Kafka→AI 服务 → 图像处理 → 结果存储 →PostgreSQL
3. **通知阶段**: AI 服务 →Kafka→Query Service→WebSocket 推送 → 用户
4. **查询阶段**: 用户 →Query Service→ 数据库/缓存 → 结果返回
5. **分选阶段**: 用户 →Sort Service→ 设备控制 → 状态监控 → 结果反馈

### 🏷️ 主要数据结构：

- **UploadResponse**: 上传响应，包含任务 ID、预签名 URL、状态
- **ResultResponse**: 完整的检测结果，包含任务信息和检测详情
- **DetectionResult**: 单个检测结果，包含物体类别、置信度、边界框、质量等级
- **QueryParams**: 查询参数，支持分页、状态筛选、时间范围
- **SortCommand**: 分选命令，包含操作类型、目标等级、设备参数
- **SortStatus**: 分选状态，包含运行状态、统计信息、设备参数

### 🔌 通信协议：

- **HTTP/HTTPS**: REST API 接口通信
- **WebSocket**: 实时双向通信
- **Kafka**: 异步事件驱动通信
- **gRPC**: 内部微服务通信（预留）

### � 部署架构：

- **微服务架构**: 服务独立部署和扩展
- **容器化**: Docker 容器部署
- **负载均衡**: 支持水平扩展
- **服务发现**: 支持动态服务注册

## �🛠️ 图表生成工具

### 使用 PlantUML 生成图表

本项目使用 PlantUML 生成所有 UML 图表。要重新生成图表：

```bash
# 安装依赖
sudo apt install openjdk-11-jre-headless

# 下载 PlantUML
wget -O plantuml.jar https://github.com/plantuml/plantuml/releases/download/v1.2024.7/plantuml-1.2024.7.jar

# 生成所有图表
./generate-diagrams.sh

# 生成特定图表
./generate-diagrams.sh sequence     # 序列图
./generate-diagrams.sh architecture # 架构图
./generate-diagrams.sh workflow     # 工作流图
./generate-diagrams.sh dataflow     # 数据流图
```

### 源文件说明

- **sequence-diagram.puml**: 序列图源文件
- **architecture-diagram.puml**: 架构图源文件
- **workflow-diagram.puml**: 工作流图源文件
- **dataflow-diagram.puml**: 数据流图源文件

### 修改图表

1. 编辑对应的 `.puml` 源文件
2. 运行 `./generate-diagrams.sh` 重新生成
3. 检查生成的 `.png` 文件

## 📋 图表使用指南

### 在文档中引用图表

```markdown
![系统架构图](uml-output/果蔬检测系统-架构图.png)
![序列图](uml-output/果蔬检测系统-序列图.png)
![工作流图](uml-output/果蔬检测系统-工作流图.png)
![数据流图](uml-output/果蔬检测系统-数据流图.png)
```

### 图表更新流程

1. 代码变更后更新对应的 `.puml` 文件
2. 运行生成脚本更新图片
3. 提交代码时一并提交更新的图表文件
4. 在 PR 中说明图表的变更内容

---

_本文档随项目代码同步更新，最后更新时间: 2025 年 10 月 4 日_

- **rudg**: Rust UML Diagram Generator，用于从 Rust 代码生成 DOT 文件
- **Graphviz/dot**: 将 DOT 文件转换为 PNG 图片
- **自定义脚本**: 修复格式问题和创建综合架构图

## 📍 文件位置

所有 UML 图形文件位于：`/home/lanshi/Documents/Projects/guoyan-zhizao/backend/uml-output/`

生成时间：2025 年 10 月 2 日

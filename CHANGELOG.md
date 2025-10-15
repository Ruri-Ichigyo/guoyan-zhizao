# 更新日志

本文档记录果蔬智造核心平台的版本更新历史。

---

## [v1.1.0] - 2025-10-15

### 🎉 新增功能

#### AI 聊天助手
- ✨ 集成 LLM（大语言模型）支持自然语言交互
- ✨ 实现 MCP（Model Context Protocol）协议
- ✨ 支持数据库查询工具调用
- ✨ 会话管理：创建、列表、删除聊天会话
- ✨ 消息历史记录和轮询获取新消息
- ✨ Redis 缓存聊天历史（最近 20 条）
- ✨ 支持 SiliconFlow 和 Anthropic Claude API

#### 统一记录 API
- ✨ 新增 `/api/v1/records` 端点，统一查询上传和检测记录
- ✨ 支持按来源、状态、时间范围过滤
- ✨ 统一的记录详情和图片获取接口
- ✨ 汇总统计信息

#### 实时检测
- ✨ 新增 `/api/v1/detections` 端点支持实时检测记录
- ✨ 支持摄像头、移动应用等多种来源
- ✨ 快照图像存储和获取

#### AI 服务增强
- ✨ 多模型支持和动态加载
- ✨ 模型注册表和元数据管理
- ✨ 模型权重缓存优化
- ✨ `/models` 和 `/models/{name}` 端点

### 🔧 改进优化

#### 数据库
- 🔧 新增 `chat_sessions` 和 `chat_messages` 表
- 🔧 新增 `detections` 表支持实时检测
- 🔧 新增 `users` 表支持用户管理
- 🔧 优化索引提升查询性能

#### 缓存策略
- 🔧 扩展 Redis 缓存用途：结果 + 会话 + 聊天历史
- 🔧 活跃会话缓存（TTL: 1 小时）
- 🔧 会话历史缓存（TTL: 30 分钟）

#### 前端界面
- 🔧 新增 ChatWidget 组件（悬浮聊天窗口）
- 🔧 新增 ChatWindow、ChatBubble、ChatMessage 组件
- 🔧 Pinia store 管理聊天状态
- 🔧 支持 Markdown 消息渲染
- 🔧 自动轮询获取新消息

#### 后端服务
- 🔧 重构为模块化服务层（LLM Client、MCP Client）
- 🔧 异步非阻塞处理提升并发能力
- 🔧 统一错误处理和日志记录
- 🔧 工具调用元数据记录

### 📚 文档更新

- 📝 新增 `CHAT_MCP.md` - AI 聊天助手与 MCP 集成文档
- 📝 更新 `API.md` - 添加聊天相关 API 端点
- 📝 更新 `PROJECT_GUIDE.md` - 更新架构和模块说明
- 📝 更新 `PORTS.md` - 添加新的环境变量说明
- 📝 更新 `DEPLOY.md` - 添加 LLM 配置说明
- 📝 更新 `README.md` - 添加聊天功能介绍

### 🐛 Bug 修复

- 🐛 修复上传记录状态更新问题
- 🐛 修复 Redis 缓存键冲突
- 🐛 修复图片内容类型判断
- 🐛 修复并发请求时的数据竞争

### ⚠️ 破坏性变更

- ⚠️ 环境变量 `LLM_API_KEY` 为聊天功能必需项
- ⚠️ 数据库需要运行新的迁移脚本（`init-db/00-schema.sql`）

### 🔒 安全更新

- 🔒 MCP 查询限制：仅允许 SELECT 语句
- 🔒 SQL 注入防护：使用 SQLx 预编译
- 🔒 表级白名单控制
- 🔒 查询超时和行数限制
- 🔒 消息长度限制（10,000 字符）

---

## [v1.0.0] - 2025-10-14

### 🎉 首次发布

#### 核心功能
- ✨ 图像上传与 AI 推理
- ✨ DenseNet121 分类模型
- ✨ A/B/C 三级质量评估
- ✨ 历史记录查询
- ✨ 统计数据展示
- ✨ 多端上传支持

#### 技术架构
- ✨ Vue 3 + Element Plus + TypeScript 前端
- ✨ Rust (Axum + SQLx) 后端
- ✨ Python (FastAPI + PyTorch) AI 服务
- ✨ PostgreSQL 15 数据库
- ✨ Redis 7 缓存
- ✨ Docker Compose 容器化部署

#### API 端点
- ✨ `POST /api/v1/uploads` - 图片上传
- ✨ `GET /api/v1/results` - 结果列表
- ✨ `GET /api/v1/results/:id` - 结果详情
- ✨ `GET /api/v1/statistics` - 统计信息
- ✨ `GET /health` - 健康检查

#### 文档
- 📝 API.md - API 接口文档
- 📝 PROJECT_GUIDE.md - 项目指南
- 📝 DEPLOY.md - 部署手册
- 📝 ALGORITHM.md - 算法说明
- 📝 PORTS.md - 端口配置

---

## 版本号说明

本项目遵循[语义化版本 2.0.0](https://semver.org/lang/zh-CN/)规范：

- **主版本号（MAJOR）**：进行不兼容的 API 更改
- **次版本号（MINOR）**：添加向后兼容的功能
- **修订号（PATCH）**：进行向后兼容的问题修正

格式：`主版本号.次版本号.修订号`

---

## 图例说明

- ✨ 新增功能
- 🔧 改进优化
- 🐛 Bug 修复
- 📚 文档更新
- ⚠️ 破坏性变更
- 🔒 安全更新
- 🗑️ 移除功能

---

**最后更新**: 2025-10-15  
**维护团队**: 果蔬智造团队

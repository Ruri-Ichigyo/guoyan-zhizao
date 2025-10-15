# 文档快速索引

本文档提供果蔬智造核心平台所有文档的快速导航和查找指南。

---

## 📚 文档列表

### 核心文档（必读）

| 文档 | 路径 | 说明 | 关键词 |
|------|------|------|--------|
| **主 README** | [README.md](../README.md) | 项目概览、快速开始 | 快速上手、环境配置、功能介绍 |
| **API 参考** | [API.md](API.md) | 完整 API 接口定义 | REST API、端点、请求响应 |
| **项目指南** | [PROJECT_GUIDE.md](PROJECT_GUIDE.md) | 架构设计、模块详解 | 系统架构、技术栈、代码结构 |
| **部署手册** | [DEPLOY.md](DEPLOY.md) | 生产环境部署 | Docker、运维、配置 |

### 专题文档

| 文档 | 路径 | 说明 | 关键词 |
|------|------|------|--------|
| **算法说明** | [ALGORITHM.md](ALGORITHM.md) | DenseNet121 模型详解 | 深度学习、训练、质量评估 |
| **AI 聊天助手** | [CHAT_MCP.md](CHAT_MCP.md) | LLM 集成、MCP 协议 | 聊天、数据库查询、工具调用 |
| **端口配置** | [PORTS.md](PORTS.md) | 服务端口说明 | 端口映射、网络配置、环境变量 |

### 变更记录

| 文档 | 路径 | 说明 | 关键词 |
|------|------|------|--------|
| **更新日志** | [CHANGELOG.md](../CHANGELOG.md) | 版本历史、更新内容 | 版本、新特性、Bug 修复 |

---

## 🔍 按主题查找

### 入门与安装

**我想快速开始使用**
- 📖 [主 README - 快速上手](../README.md#快速上手)
- 📖 [部署手册 - 快速启动](DEPLOY.md#快速启动)

**我想了解环境要求**
- 📖 [部署手册 - 环境要求](DEPLOY.md#环境要求)
- 📖 [主 README - 环境准备](../README.md#环境准备)

**我想进行本地开发**
- 📖 [主 README - 本地开发模式](../README.md#本地开发模式)
- 📖 [项目指南 - 启动与开发流程](PROJECT_GUIDE.md#启动与开发流程)

### 系统架构

**我想了解整体架构**
- 📖 [项目指南 - 系统架构](PROJECT_GUIDE.md#系统架构)
- 📖 [主 README - 架构速览](../README.md#架构速览)

**我想了解服务拓扑**
- 📖 [项目指南 - 服务拓扑](PROJECT_GUIDE.md#服务拓扑)
- 📖 [端口配置 - 服务详细说明](PORTS.md#服务详细说明)

**我想了解数据流程**
- 📖 [项目指南 - 核心流程](PROJECT_GUIDE.md#核心流程)
- 📖 [AI 聊天助手 - 数据流](CHAT_MCP.md#数据流)

### API 开发

**我想调用上传接口**
- 📖 [API 参考 - 图像上传与分析](API.md#图像上传与分析)

**我想查询检测结果**
- 📖 [API 参考 - 统一记录接口](API.md#统一记录接口)
- 📖 [API 参考 - 遗留接口](API.md#遗留接口)

**我想获取统计数据**
- 📖 [API 参考 - 统计接口](API.md#统计接口)

**我想集成聊天功能**
- 📖 [API 参考 - AI 聊天接口](API.md#ai-聊天接口)
- 📖 [AI 聊天助手 - API 接口详解](CHAT_MCP.md#api-接口详解)

### 后端开发

**我想了解 Rust 后端结构**
- 📖 [项目指南 - Rust 后端](PROJECT_GUIDE.md#rust-后端)

**我想了解数据库表结构**
- 📖 [项目指南 - PostgreSQL 表结构](PROJECT_GUIDE.md#postgresql-表结构)
- 📖 [AI 聊天助手 - 数据库表结构](CHAT_MCP.md#数据库表结构)

**我想了解缓存策略**
- 📖 [项目指南 - Redis 缓存策略](PROJECT_GUIDE.md#redis-缓存策略)
- 📖 [AI 聊天助手 - Redis 缓存策略](CHAT_MCP.md#redis-缓存策略)

**我想添加新的 API 端点**
- 📖 [项目指南 - 业务路由](PROJECT_GUIDE.md#rust-后端)
- 📖 [API 参考](API.md)

### AI 与算法

**我想了解分类模型**
- 📖 [算法说明 - DenseNet121 分类器](ALGORITHM.md#densenet121-分类器)

**我想了解质量评估**
- 📖 [算法说明 - 质量评估模块](ALGORITHM.md#质量评估模块)

**我想训练自己的模型**
- 📖 [算法说明 - 模型训练流程](ALGORITHM.md#模型训练流程)
- 📖 [算法说明 - 数据集准备](ALGORITHM.md#数据集准备)

**我想了解 AI 服务**
- 📖 [项目指南 - AI 推理服务](PROJECT_GUIDE.md#ai-推理服务)
- 📖 [API 参考 - AI 服务 API](API.md#ai-服务-fastapi--pytorch)

### 聊天与 MCP

**我想了解聊天功能**
- 📖 [AI 聊天助手 - 概览](CHAT_MCP.md#概览)
- 📖 [主 README - AI 助手](../README.md#核心功能)

**我想了解 MCP 协议**
- 📖 [AI 聊天助手 - MCP 协议实现](CHAT_MCP.md#mcp-协议实现)

**我想配置 LLM**
- 📖 [AI 聊天助手 - LLM 集成](CHAT_MCP.md#llm-集成)
- 📖 [项目指南 - 配置与环境变量](PROJECT_GUIDE.md#配置与环境变量)

**我想添加新工具**
- 📖 [AI 聊天助手 - 添加新工具](CHAT_MCP.md#添加新工具)

### 前端开发

**我想了解前端结构**
- 📖 [项目指南 - 前端应用](PROJECT_GUIDE.md#前端应用)

**我想了解聊天组件**
- 📖 [项目指南 - 核心组件](PROJECT_GUIDE.md#前端应用)
- 📖 [AI 聊天助手 - 使用示例](CHAT_MCP.md#使用示例)

### 部署与运维

**我想部署到生产环境**
- 📖 [部署手册 - Docker Compose 部署](DEPLOY.md#docker-compose-部署)

**我想了解资源需求**
- 📖 [部署手册 - 组件与资源建议](DEPLOY.md#组件与资源建议)

**我想配置环境变量**
- 📖 [项目指南 - 配置与环境变量](PROJECT_GUIDE.md#配置与环境变量)
- 📖 [端口配置 - 环境变量](PORTS.md#环境变量)

**我想了解端口映射**
- 📖 [端口配置 - 端口总览](PORTS.md#端口总览)

**我想进行故障排查**
- 📖 [AI 聊天助手 - 故障排查](CHAT_MCP.md#故障排查)

### 测试与质量

**我想运行测试**
- 📖 [项目指南 - 测试与质量保障](PROJECT_GUIDE.md#测试与质量保障)

**我想查看日志**
- 📖 [项目指南 - 日志与监控](PROJECT_GUIDE.md#日志与监控)

---

## 🎯 按角色查找

### 新手开发者

1. 📖 [主 README](../README.md) - 了解项目
2. 📖 [项目指南](PROJECT_GUIDE.md) - 理解架构
3. 📖 [API 参考](API.md) - 熟悉接口

### 前端开发者

1. 📖 [API 参考](API.md) - API 接口
2. 📖 [项目指南 - 前端应用](PROJECT_GUIDE.md#前端应用)
3. 📖 [端口配置](PORTS.md) - 环境配置

### 后端开发者

1. 📖 [项目指南 - Rust 后端](PROJECT_GUIDE.md#rust-后端)
2. 📖 [API 参考](API.md) - 接口设计
3. 📖 [项目指南 - 数据与存储](PROJECT_GUIDE.md#数据与存储)
4. 📖 [AI 聊天助手](CHAT_MCP.md) - 聊天服务

### 算法工程师

1. 📖 [算法说明](ALGORITHM.md) - 完整阅读
2. 📖 [项目指南 - AI 推理服务](PROJECT_GUIDE.md#ai-推理服务)

### AI 集成开发者

1. 📖 [AI 聊天助手](CHAT_MCP.md) - 完整阅读
2. 📖 [API 参考 - AI 聊天接口](API.md#ai-聊天接口)
3. 📖 [项目指南 - 聊天服务](PROJECT_GUIDE.md#rust-后端)

### 运维工程师

1. 📖 [部署手册](DEPLOY.md) - 完整阅读
2. 📖 [端口配置](PORTS.md) - 端口映射
3. 📖 [项目指南 - 配置与环境变量](PROJECT_GUIDE.md#配置与环境变量)

### 架构师

1. 📖 [项目指南](PROJECT_GUIDE.md) - 完整阅读
2. 📖 [API 参考](API.md) - 接口设计
3. 📖 [AI 聊天助手](CHAT_MCP.md) - MCP 架构

---

## 🔧 按任务查找

### 配置类

| 任务 | 文档位置 |
|------|---------|
| 配置数据库连接 | [项目指南 - 配置与环境变量](PROJECT_GUIDE.md#配置与环境变量) |
| 配置 Redis 缓存 | [项目指南 - 配置与环境变量](PROJECT_GUIDE.md#配置与环境变量) |
| 配置 LLM API | [AI 聊天助手 - 配置 LLM](CHAT_MCP.md#配置-llm) |
| 配置端口映射 | [端口配置](PORTS.md) |

### 开发类

| 任务 | 文档位置 |
|------|---------|
| 添加新的 API 端点 | [项目指南 - Rust 后端](PROJECT_GUIDE.md#rust-后端) |
| 添加前端页面 | [项目指南 - 前端应用](PROJECT_GUIDE.md#前端应用) |
| 添加数据库表 | [项目指南 - PostgreSQL 表结构](PROJECT_GUIDE.md#postgresql-表结构) |
| 添加 MCP 工具 | [AI 聊天助手 - 添加新工具](CHAT_MCP.md#添加新工具) |
| 训练新模型 | [算法说明 - 模型训练流程](ALGORITHM.md#模型训练流程) |

### 运维类

| 任务 | 文档位置 |
|------|---------|
| 部署到生产环境 | [部署手册](DEPLOY.md) |
| 查看服务日志 | [项目指南 - 日志与监控](PROJECT_GUIDE.md#日志与监控) |
| 故障排查 | [AI 聊天助手 - 故障排查](CHAT_MCP.md#故障排查) |
| 性能优化 | [部署手册 - 组件与资源建议](DEPLOY.md#组件与资源建议) |

### 集成类

| 任务 | 文档位置 |
|------|---------|
| 调用上传 API | [API 参考 - 图像上传](API.md#图像上传与分析) |
| 集成聊天功能 | [AI 聊天助手 - API 接口详解](CHAT_MCP.md#api-接口详解) |
| 查询检测结果 | [API 参考 - 统一记录接口](API.md#统一记录接口) |

---

## 📖 文档阅读顺序建议

### 快速入门路径（30 分钟）

1. ⏱️ 5 分钟：[主 README - 核心功能](../README.md#核心功能)
2. ⏱️ 10 分钟：[主 README - 快速上手](../README.md#快速上手)
3. ⏱️ 15 分钟：[API 参考 - 核心接口](API.md#核心功能接口)

### 全面了解路径（2 小时）

1. ⏱️ 20 分钟：[主 README](../README.md) - 完整阅读
2. ⏱️ 30 分钟：[项目指南 - 系统架构](PROJECT_GUIDE.md#系统架构)
3. ⏱️ 30 分钟：[API 参考](API.md) - 浏览所有端点
4. ⏱️ 20 分钟：[算法说明](ALGORITHM.md) - 了解 AI 模型
5. ⏱️ 20 分钟：[AI 聊天助手](CHAT_MCP.md) - 了解聊天功能

### 深度学习路径（1 天）

1. ⏱️ 1 小时：全面了解路径
2. ⏱️ 2 小时：[项目指南](PROJECT_GUIDE.md) - 完整阅读
3. ⏱️ 1 小时：[部署手册](DEPLOY.md) - 完整阅读
4. ⏱️ 1 小时：[AI 聊天助手](CHAT_MCP.md) - 完整阅读
5. ⏱️ 1 小时：[算法说明](ALGORITHM.md) - 完整阅读
6. ⏱️ 2 小时：实际操作和测试

---

## 🆘 常见问题快速链接

| 问题 | 解决方案 |
|------|---------|
| 服务启动失败 | [部署手册 - 快速启动](DEPLOY.md#快速启动) |
| API 调用 404 | [API 参考 - 基础信息](API.md#基础信息) |
| 数据库连接失败 | [端口配置 - PostgreSQL](PORTS.md#postgresql) |
| Redis 连接失败 | [端口配置 - Redis](PORTS.md#redis) |
| 聊天功能无响应 | [AI 聊天助手 - 故障排查](CHAT_MCP.md#故障排查) |
| 模型加载失败 | [算法说明 - DenseNet121](ALGORITHM.md#densenet121-分类器) |

---

## 📝 文档贡献

欢迎改进文档：

1. 发现错误或不清晰之处 → 提交 Issue
2. 有改进建议 → 在 Issue 中讨论
3. 直接改进 → 提交 Pull Request

**文档风格指南**：
- 使用清晰的标题层级
- 提供代码示例
- 添加图表说明（如适用）
- 保持简洁明了
- 更新"最后更新"日期

---

**最后更新**: 2025-10-15  
**版本**: v1.1.0  
**维护团队**: 果蔬智造团队

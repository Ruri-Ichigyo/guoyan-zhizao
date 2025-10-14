# 更新日志

本文档记录了果蔬智造核心平台的主要版本变更和功能更新。

## [1.0.0] - 2025-10-14

### 项目归档与整理

**状态**: 生产就绪

这是项目的第一个正式版本，标志着所有核心功能已完成开发、测试并准备投入生产使用。

#### 新增功能

- ✅ **AI 聊天助手模块**
  - 前端悬浮球聊天组件
  - 后端聊天会话管理 API
  - 集成 SiliconFlow LLM API（Tencent Hunyuan-MT-7B 模型）
  - MCP（Model Context Protocol）数据库查询能力
  - 支持自然语言查询识别统计和历史记录

- ✅ **用户管理模块**
  - 用户表结构设计和实现
  - 用户角色系统（admin、user、operator）
  - 用户状态管理（active、inactive、suspended）
  - 所有数据关联用户 ID

- ✅ **统一记录接口**
  - 融合上传记录与实时检测的统一查询 API
  - 支持多种数据源筛选（upload、webcam、mobile）
  - 支持状态筛选和分页查询

- ✅ **数据库优化**
  - 完整的数据库 schema 设计
  - 聊天会话和消息表
  - MCP 只读用户权限配置
  - 外键约束和索引优化（27 个索引）

#### 改进

- 🔧 **文档系统重构**
  - 创建文档中心索引 ([docs/README.md](docs/README.md))
  - 更新主 README 反映所有功能
  - 归档项目规范文档到 `docs/archive/specifications/`
  - 完善 API 文档，添加聊天接口说明

- 🔧 **项目结构优化**
  - 清理临时文件和构建缓存
  - 删除过时的代码和文档
  - 规范化目录结构
  - 添加归档说明文档

- 🔧 **代码质量提升**
  - 后端添加聊天服务层（LLM 客户端、MCP 客户端）
  - 前端添加聊天 API 封装
  - 完善错误处理和日志记录

#### 测试

- ✅ **集成测试覆盖**
  - 数据库集成测试（4/4 通过）
  - MCP 用户权限测试（8/8 通过）
  - 数据关系完整性测试（4/4 通过）
  - 复杂查询测试（3/3 通过）
  - 性能测试（2/2 通过）

- ✅ **API 功能测试**
  - 聊天会话创建和管理
  - AI 消息响应
  - MCP 数据库查询功能

#### 文档

- 📝 创建 [CHANGELOG.md](CHANGELOG.md) - 更新日志
- 📝 创建 [PROJECT_STATUS.md](PROJECT_STATUS.md) - 项目状态报告
- 📝 更新 [README.md](README.md) - 主文档
- 📝 创建 [docs/README.md](docs/README.md) - 文档中心
- 📝 创建 [docs/archive/specifications/README.md](docs/archive/specifications/README.md) - 规范文档归档说明

#### 移除

- 🗑️ 删除 `.qwen/` - 其他 AI 助手配置
- 🗑️ 删除 `DATABASE_INTEGRATION.md` - 临时整理文档
- 🗑️ 删除 `frontend/front.md` - 过时的前端说明
- 🗑️ 删除 `ai/src/image_classification/` 和 `ai/src/image_classification_tf_1.8.0/` - 旧版图像分类代码
- 🗑️ 删除 `ai/.python-version` - Python 版本固定文件
- 🗑️ 删除 `ai/.claude/` - 冗余的 Claude 配置
- 🗑️ 清理 Python 和 Rust 构建缓存
- 🗑️ 清理日志文件

---

## [0.3.0] - 2025-10-13

### 数据库整合与用户管理

#### 新增

- 用户管理模块基础架构
- 聊天会话和消息表设计
- MCP 只读用户权限配置

#### 改进

- 统一数据库 schema
- 完善数据关系设计
- 添加数据库验证脚本

---

## [0.2.0] - 2025-10-12

### 统一记录接口

#### 新增

- 统一记录查询 API
- 支持多数据源筛选
- 分页和排序功能

#### 改进

- 优化查询性能
- 添加 Redis 缓存层
- 完善 API 文档

---

## [0.1.0] - 2025-10-11

### 初始版本

#### 核心功能

- 果蔬图像识别（DenseNet121）
- 质量分级系统（A/B/C 三级）
- 图片上传和结果查询
- 实时检测记录
- 统计分析和数据可视化

#### 技术架构

- 前端：Vue 3 + Element Plus
- 后端：Rust (Axum + SQLx)
- AI 服务：Python (FastAPI + PyTorch)
- 数据库：PostgreSQL 15
- 缓存：Redis 7
- 容器化：Docker + Docker Compose

---

## 版本说明

版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/) 规范：

- **主版本号（Major）**：重大架构变更或不兼容的 API 修改
- **次版本号（Minor）**：新功能添加，保持向后兼容
- **修订号（Patch）**：问题修复和小的改进

## 未来规划

### v1.1.0（计划中）

- [ ] JWT 认证中间件完善
- [ ] 用户注册和登录 API
- [ ] 密码重置功能
- [ ] WebSocket 实时通信（替代 HTTP 轮询）
- [ ] 聊天历史导出功能

### v1.2.0（规划中）

- [ ] 多语言支持（英文）
- [ ] 高级统计分析面板
- [ ] 批量图片处理
- [ ] 模型版本管理
- [ ] API 速率限制和配额管理

---

**维护**: 果蔬智造团队  
**最后更新**: 2025-10-14

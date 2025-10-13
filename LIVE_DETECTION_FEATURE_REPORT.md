# 实时监控识别记录功能实现报告

## 功能概述
实现了前端实时监控系统中，当识别到果蔬时自动截屏并保存到数据库，并在前端显示识别记录列表的完整功能。

## 实现的功能

### 1. 数据库模型设计
- **文件**: `backend/src/db.rs`
- **表名**: `detections`
- **字段**:
  - `id`: UUID主键
  - `snapshot`: BYTEA (存储截图二进制数据)
  - `labels`: TEXT[] (识别标签数组)
  - `confidence_scores`: REAL[] (置信度分数数组)
  - `detected_classes`: TEXT[] (检测到的类别数组)
  - `created_at`: TIMESTAMPTZ (创建时间)
  - `source`: TEXT (来源，默认为'live_camera')

### 2. 后端API接口
- **文件**: `backend/src/routes/detections.rs`
- **接口列表**:
  - `POST /api/v1/detections` - 创建新的识别记录
  - `GET /api/v1/detections` - 获取识别记录列表（支持分页）
  - `GET /api/v1/detections/:id` - 获取单个识别记录详情
  - `GET /api/v1/detections/:id/snapshot` - 获取识别记录的截图

### 3. 数据模型
- **文件**: `backend/src/models.rs`
- **新增模型**:
  - `LiveDetection`: 完整的识别记录
  - `LiveDetectionSummary`: 识别记录摘要（不包含二进制图片数据）
  - `CreateLiveDetectionRequest`: 创建识别记录的请求结构

### 4. 前端API接口
- **文件**: `frontend/src/api/detection.ts`
- **功能**: 封装了所有检测记录相关的API调用

### 5. 自动截图功能
- **位置**: `frontend/src/components/LiveCamera.vue`
- **功能**:
  - 新增`autoCapture`开关，默认启用
  - 新增`captureThreshold`配置识别置信度阈值（默认0.7）
  - 实现了`startAutoCapture()`和`stopAutoCapture()`方法
  - 实现了`checkAndCaptureDetection()`方法，每2秒检查一次是否有检测结果
  - 当检测到置信度超过阈值的果蔬时，自动截图并保存到数据库

### 6. 识别记录列表页面
- **文件**: `frontend/src/views/DetectionsView.vue`
- **功能**:
  - 显示所有识别记录的列表
  - 支持分页（10/20/50/100条每页）
  - 显示缩略图预览（点击可放大）
  - 显示识别结果（标签和置信度）
  - 显示检测类别
  - 显示时间和来源
  - 支持查看详情（弹窗显示完整信息）
  - 支持下载截图

### 7. 路由和导航
- **文件**:
  - `frontend/src/router/index.ts` - 添加了`/detections`路由
  - `frontend/src/App.vue` - 在侧边栏添加了"识别记录"菜单项

## 技术栈

### 后端
- **语言**: Rust
- **框架**: Axum
- **数据库**: PostgreSQL (使用SQLx)
- **功能**: RESTful API, Base64图片编码/解码

### 前端
- **框架**: Vue 3 (TypeScript)
- **UI库**: Element Plus
- **功能**:
  - 实时视频流（WebRTC/getUserMedia）
  - 自动截图
  - 图片预览
  - 分页列表
  - 文件下载

## 工作流程

1. **启动监控**: 用户在实时摄像页面启动摄像头
2. **自动检测**: 系统每2秒检查一次是否有果蔬被识别
3. **自动截图**: 当检测到置信度≥70%的果蔬时，自动截取当前帧
4. **保存记录**: 将截图（Base64编码）和识别信息发送到后端API
5. **数据存储**: 后端将数据存储到PostgreSQL数据库
6. **查看记录**: 用户可在"识别记录"页面查看所有历史记录

## 下一步优化建议

1. **AI集成**:
   - 当前使用的是模拟检测数据
   - 需要集成实际的AI推理服务（可能通过WebSocket实时获取检测结果）

2. **性能优化**:
   - 图片压缩：在前端压缩截图后再上传
   - 懒加载：列表中的图片使用懒加载
   - 缓存：使用Redis缓存热门查询

3. **功能增强**:
   - 添加过滤和搜索功能
   - 支持批量删除
   - 添加统计图表
   - 导出Excel报表

4. **安全性**:
   - 添加身份验证和授权
   - 限制API访问频率
   - 添加数据加密

## 文件清单

### 后端文件
- `backend/src/db.rs` - 数据库初始化和表创建
- `backend/src/models.rs` - 数据模型定义
- `backend/src/errors.rs` - 错误类型定义
- `backend/src/routes/detections.rs` - API路由处理
- `backend/src/routes/mod.rs` - 路由模块导出
- `backend/src/main.rs` - 主程序和路由注册

### 前端文件
- `frontend/src/api/detection.ts` - API调用接口
- `frontend/src/views/DetectionsView.vue` - 识别记录列表页面
- `frontend/src/components/LiveCamera.vue` - 实时摄像头组件（已更新）
- `frontend/src/router/index.ts` - 路由配置（已更新）
- `frontend/src/App.vue` - 主应用组件（已更新）

## 测试建议

1. **单元测试**: 测试API端点的正确性
2. **集成测试**: 测试前后端数据流
3. **性能测试**: 测试大量数据下的查询性能
4. **浏览器兼容性测试**: 测试不同浏览器的摄像头访问

## 总结

本次实现完整地完成了实时监控识别记录功能，包括：
- ✅ 数据库表设计
- ✅ 后端API接口
- ✅ 前端API封装
- ✅ 自动截图功能
- ✅ 识别记录列表展示
- ✅ 图片预览和下载

系统已经具备了基本的实时监控和记录功能，可以在此基础上进行进一步的优化和扩展。

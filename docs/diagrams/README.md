# 系统架构图和流程图

本目录包含果蔬智造核心平台 v1.1.0 的系统架构图和流程图源文件（PlantUML 格式）。

## 📋 图表文件列表

| 文件名 | 说明 | 内容 |
|--------|------|------|
| `system-architecture.puml` | 系统架构图 | 完整的系统架构，包含前端、后端、AI服务、聊天功能、MCP集成、数据库和缓存 |
| `upload-flow.puml` | 图像上传流程图 | 从用户上传到AI识别完成的完整时序流程 |
| `chat-flow.puml` | AI聊天流程图 | 聊天助手的完整交互流程，包含LLM调用和MCP数据库查询 |
| `data-flow.puml` | 数据流图 | 数据在各组件间的流动和转换过程 |

## 🎨 生成图表

### 方法一：使用本地 PlantUML（推荐）

**前提条件**：
- Java 11 或更高版本
- PlantUML JAR 文件（脚本会自动下载）

**步骤**：
```bash
# 进入目录
cd docs/diagrams

# 运行生成脚本
./generate-updated-diagrams.sh

# 生成 SVG 矢量图（可选）
./generate-updated-diagrams.sh --svg
```

**安装 Java**：
```bash
# macOS
brew install openjdk@11

# Ubuntu/Debian
sudo apt install openjdk-11-jdk

# CentOS/RHEL
sudo yum install java-11-openjdk
```

### 方法二：使用在线工具

如果无法安装 Java，可以使用在线 PlantUML 编辑器：

1. **PlantUML 官方编辑器**
   - 访问: https://www.plantuml.com/plantuml/uml/
   - 复制 `.puml` 文件内容
   - 粘贴到编辑器
   - 点击 "Submit" 生成图片
   - 右键保存图片

2. **PlantText**
   - 访问: https://www.planttext.com/
   - 复制粘贴 `.puml` 文件内容
   - 实时预览
   - 导出为 PNG/SVG

3. **VS Code 插件**（最方便）
   - 安装插件: "PlantUML"（作者：jebbs）
   - 打开 `.puml` 文件
   - 按 `Alt+D` 预览
   - 右键 → "Export Current Diagram" 导出

### 方法三：使用 Docker

```bash
# 拉取 PlantUML Docker 镜像
docker pull plantuml/plantuml-server

# 运行服务器
docker run -d -p 8080:8080 plantuml/plantuml-server

# 访问 http://localhost:8080 使用 Web 界面
```

## 📊 生成的图表

图表将保存在 `output/` 目录：

```
output/
├── system-architecture.png  # 系统架构图
├── upload-flow.png          # 上传流程图
├── chat-flow.png            # 聊天流程图
└── data-flow.png            # 数据流图
```

## 🔧 自定义图表

### 编辑源文件

1. 使用文本编辑器打开 `.puml` 文件
2. 修改 PlantUML 代码
3. 重新生成图表

### PlantUML 语法参考

- 官方文档: https://plantuml.com/zh/
- 快速参考: https://plantuml.com/zh/guide
- 序列图: https://plantuml.com/zh/sequence-diagram
- 组件图: https://plantuml.com/zh/component-diagram

### 常用元素

```plantuml
' 定义组件
rectangle "组件名" as Alias
database "数据库" as DB
cloud "云服务" as Cloud

' 连接关系
Component1 --> Component2 : 说明文字
Component1 -[#red]-> Component2 : 红色箭头

' 分组
package "包名" {
    [组件A]
    [组件B]
}

' 注释
note right of Component
  这是一个注释
end note
```

## 🎯 图表说明

### 1. 系统架构图 (system-architecture.puml)

**展示内容**：
- ✅ 完整的系统组件层次
- ✅ 前端服务（Vue 3 + Nginx）
- ✅ 后端服务（Rust + Axum）
- ✅ AI 推理服务（FastAPI + PyTorch）
- ✅ 聊天服务（LLM + MCP）
- ✅ 数据存储（PostgreSQL + Redis）
- ✅ 外部服务（LLM API）
- ✅ 所有组件间的连接关系

**适用场景**：
- 向新成员介绍系统
- 技术方案评审
- 文档和演示

### 2. 图像上传流程图 (upload-flow.puml)

**展示内容**：
- ✅ 用户上传图片的完整时序
- ✅ 后端处理流程
- ✅ AI 推理过程
- ✅ 数据持久化和缓存
- ✅ 响应返回
- ✅ 性能指标和时间估算

**适用场景**：
- 理解上传流程
- 性能优化分析
- 排查上传问题

### 3. AI 聊天流程图 (chat-flow.puml)

**展示内容**：
- ✅ 会话创建和初始化
- ✅ 用户消息处理
- ✅ LLM 调用和工具决策
- ✅ MCP 数据库查询
- ✅ 响应生成和返回
- ✅ 轮询获取新消息
- ✅ 安全限制和性能指标

**适用场景**：
- 理解聊天功能
- 集成 LLM 服务
- MCP 协议实现
- 故障排查

### 4. 数据流图 (data-flow.puml)

**展示内容**：
- ✅ 数据源和输入
- ✅ 数据传输格式
- ✅ 数据处理和转换
- ✅ 数据存储策略
- ✅ 数据输出格式
- ✅ 数据展示方式

**适用场景**：
- 理解数据流向
- 数据格式设计
- 性能优化
- 数据一致性分析

## 📈 版本历史

### v1.1.0 (2025-10-15)
- ✨ 新增 AI 聊天流程图
- ✨ 更新系统架构图（包含 LLM 和 MCP）
- ✨ 更新数据流图（包含聊天数据流）
- 🔧 改进图表样式和布局
- 📝 完善图表注释和说明

### v1.0.0 (2025-10-14)
- ✨ 初始版本
- ✨ 系统架构图
- ✨ 上传流程图

## 🤝 贡献指南

欢迎改进图表：

1. 编辑 `.puml` 源文件
2. 测试生成效果
3. 更新此 README
4. 提交 Pull Request

## 📞 技术支持

如有问题，请：
- 提交 Issue
- 查看 PlantUML 官方文档
- 联系项目维护团队

---

**最后更新**: 2025-10-15  
**版本**: v1.1.0  
**维护团队**: 果蔬智造团队

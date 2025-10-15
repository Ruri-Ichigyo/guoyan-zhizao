# 图表输出目录

本目录用于存放生成的架构图和流程图。

## 📊 应包含的文件

生成后，此目录应包含以下图片：

- `system-architecture.png` - 系统架构图 v1.1.0
- `upload-flow.png` - 图像上传流程图
- `chat-flow.png` - AI 聊天流程图
- `data-flow.png` - 数据流图

## 🎨 生成图片

请使用以下任一方法生成图片：

### 方法 1: VS Code（推荐）

1. 安装 PlantUML 插件
2. 打开 `.puml` 文件
3. 按 `Alt+D` 预览
4. 右键 → "Export Current Diagram"

### 方法 2: 在线工具

1. 访问 https://www.plantuml.com/plantuml/uml/
2. 复制 `.puml` 文件内容
3. 粘贴并生成
4. 下载图片到此目录

### 方法 3: 生成脚本

```bash
cd ..
./generate-updated-diagrams.sh
```

## 📝 注意事项

- 图片应为 PNG 格式
- 推荐分辨率：1920x1080 或更高
- 文件名应与上述列表一致
- 生成后请验证图片显示正常

---

**状态**: 等待生成  
**最后更新**: 2025-10-15

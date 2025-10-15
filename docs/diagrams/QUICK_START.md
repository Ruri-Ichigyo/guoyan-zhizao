# 快速生成架构图指南

无需安装任何软件，使用在线工具即可生成架构图！

## 🚀 最快方式：使用在线编辑器

### 步骤 1: 选择在线工具

推荐使用以下任一工具：

1. **PlantUML 官方服务器** (推荐)
   - 网址: https://www.plantuml.com/plantuml/uml/
   - 优点: 官方服务，稳定可靠
   - 支持: PNG, SVG, TXT

2. **PlantText**
   - 网址: https://www.planttext.com/
   - 优点: 实时预览，界面友好
   - 支持: PNG, SVG

3. **PlantUML Online Editor**
   - 网址: http://www.plantuml.com/plantuml/
   - 优点: 功能完整
   - 支持: PNG, SVG, PDF

### 步骤 2: 复制源代码

打开项目中的 `.puml` 文件，复制全部内容：

**系统架构图**:
```bash
cat docs/diagrams/system-architecture.puml
```

**上传流程图**:
```bash
cat docs/diagrams/upload-flow.puml
```

**聊天流程图**:
```bash
cat docs/diagrams/chat-flow.puml
```

**数据流图**:
```bash
cat docs/diagrams/data-flow.puml
```

### 步骤 3: 生成图表

1. 访问选择的在线工具
2. 粘贴 `.puml` 文件内容
3. 点击 "Submit" 或等待自动渲染
4. 右键保存图片，或点击下载按钮

## 📱 使用 PlantUML 官方服务器详细步骤

### 方法 1: Web 界面

1. **访问**: https://www.plantuml.com/plantuml/uml/

2. **输入代码**: 将 `.puml` 文件内容粘贴到文本框

3. **生成图表**: 点击 "Submit" 按钮

4. **保存图片**: 
   - PNG: 右键 → 图片另存为
   - SVG: 点击 "SVG" 链接
   - 其他格式: 选择相应链接

### 方法 2: 使用 URL 编码（高级）

PlantUML 支持通过 URL 直接生成图表：

```
https://www.plantuml.com/plantuml/png/[encoded_code]
```

但这需要特殊编码，建议使用 Web 界面。

## 🎯 推荐工作流

### 对于首次使用者

```
1. 打开 PlantUML 官方网站
   ↓
2. 复制 system-architecture.puml 内容
   ↓
3. 粘贴到网页编辑器
   ↓
4. 点击 Submit 生成图片
   ↓
5. 右键保存到 docs/diagrams/output/
   ↓
6. 重复步骤 2-5 生成其他图表
```

### 对于开发者

```
1. 安装 VS Code PlantUML 插件
   ↓
2. 打开 .puml 文件
   ↓
3. 按 Alt+D 预览
   ↓
4. 右键 → Export Current Diagram
   ↓
5. 选择 PNG 或 SVG
   ↓
6. 自动保存到 output 目录
```

## 🔧 各工具对比

| 工具 | 优点 | 缺点 | 推荐度 |
|------|------|------|--------|
| **PlantUML 官方** | 稳定、功能全 | 界面略显简陋 | ⭐⭐⭐⭐⭐ |
| **PlantText** | 实时预览、美观 | 偶尔不稳定 | ⭐⭐⭐⭐ |
| **VS Code 插件** | 最方便、离线可用 | 需要安装 | ⭐⭐⭐⭐⭐ |
| **Docker PlantUML** | 本地运行 | 需要 Docker | ⭐⭐⭐ |

## 📝 生成清单

按顺序生成以下图表：

- [ ] `system-architecture.puml` → `system-architecture.png`
- [ ] `upload-flow.puml` → `upload-flow.png`
- [ ] `chat-flow.puml` → `chat-flow.png`
- [ ] `data-flow.puml` → `data-flow.png`

保存位置: `docs/diagrams/output/`

## 💡 技巧和窍门

### 提高图片质量

1. **使用 SVG 格式**: 矢量图，无限缩放不失真
2. **调整 DPI**: 在 PlantUML 代码中添加 `skinparam dpi 300`
3. **选择合适尺寸**: 根据使用场景调整

### 批量生成

如果需要生成多个图表：

**使用脚本** (需要 Java):
```bash
cd docs/diagrams
./generate-updated-diagrams.sh
```

**使用 VS Code** (推荐):
```
Cmd+Shift+P → "PlantUML: Export Workspace Diagrams"
```

### 自定义样式

在 `.puml` 文件开头添加：

```plantuml
!theme cerulean
skinparam backgroundColor #FFFFFF
skinparam defaultFontName "Arial"
skinparam dpi 300
```

## ❓ 常见问题

### Q: 生成的图片中文显示为方块？
A: 在代码开头添加：
```plantuml
skinparam defaultFontName "Microsoft YaHei"
```

### Q: 图表太大，显示不全？
A: 在代码开头添加：
```plantuml
skinparam maxMessageSize 200
scale 0.8
```

### Q: 想修改颜色？
A: 修改 `.puml` 文件中的颜色代码，如 `#E3F2FD`

### Q: 在线工具无法访问？
A: 尝试：
1. 检查网络连接
2. 使用 VPN（如果需要）
3. 切换到其他在线工具
4. 使用 VS Code 插件离线生成

## 🎓 学习资源

- **PlantUML 中文文档**: https://plantuml.com/zh/
- **快速参考**: https://plantuml.com/zh/guide
- **示例库**: https://real-world-plantuml.com/

## ⚡ 5分钟快速开始

```bash
# 1. 复制源代码
cat docs/diagrams/system-architecture.puml | pbcopy

# 2. 打开浏览器
open https://www.plantuml.com/plantuml/uml/

# 3. 粘贴代码（Cmd+V）
# 4. 点击 Submit
# 5. 右键保存图片

# 完成！重复上述步骤生成其他图表
```

## 📧 需要帮助？

- 查看详细文档: [docs/diagrams/README.md](README.md)
- VS Code 使用指南: [docs/diagrams/VSCODE_GUIDE.md](VSCODE_GUIDE.md)
- 提交 Issue: [GitHub Issues](../../issues)

---

**最快方式**: 使用 VS Code PlantUML 插件，按 Alt+D 即可预览！

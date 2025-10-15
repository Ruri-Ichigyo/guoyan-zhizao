# 使用 VS Code 生成架构图

如果你使用 VS Code 开发，这是最简单的生成架构图的方法。

## 安装插件

1. 打开 VS Code
2. 按 `Cmd+Shift+X` (macOS) 或 `Ctrl+Shift+X` (Windows/Linux) 打开扩展面板
3. 搜索 "PlantUML"
4. 安装 **PlantUML** 插件（作者：jebbs）

![PlantUML Plugin](https://raw.githubusercontent.com/qjebbs/vscode-plantuml/master/images/icon.png)

## 安装依赖

### 选项 1: 使用 Java + Graphviz（本地渲染）

```bash
# macOS
brew install graphviz
brew install openjdk@11

# Ubuntu/Debian
sudo apt install graphviz openjdk-11-jdk

# Windows (使用 Chocolatey)
choco install graphviz jdk11
```

### 选项 2: 使用在线服务器（无需本地依赖）

插件默认使用在线服务器，无需安装任何依赖。

## 使用方法

### 预览图表

1. 在 VS Code 中打开任意 `.puml` 文件
2. 按 `Alt+D` 或使用命令面板 `PlantUML: Preview Current Diagram`
3. 图表会在侧边栏实时显示

### 导出图表

**方法 1: 右键菜单**
1. 在 `.puml` 文件中右键
2. 选择 "PlantUML: Export Current Diagram"
3. 选择格式（PNG/SVG/PDF）

**方法 2: 命令面板**
1. 按 `Cmd+Shift+P` (macOS) 或 `Ctrl+Shift+P` (Windows/Linux)
2. 输入 "PlantUML: Export"
3. 选择要导出的格式

**方法 3: 批量导出**
```bash
# 在 docs/diagrams 目录
Cmd+Shift+P → "PlantUML: Export Workspace Diagrams"
```

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Alt+D` | 预览当前图表 |
| `Ctrl+Shift+V` | 在新窗口预览 |
| 无默认快捷键 | 导出图表（使用命令面板） |

## 配置插件

在 VS Code 设置中 (`settings.json`)：

```json
{
  // 使用在线服务器（推荐，无需本地依赖）
  "plantuml.server": "https://www.plantuml.com/plantuml",
  
  // 或使用本地渲染（需要 Java + Graphviz）
  "plantuml.render": "Local",
  
  // 导出格式
  "plantuml.exportFormat": "png",
  
  // 导出输出目录
  "plantuml.exportOutDir": "output",
  
  // 包含路径（如果使用 !include）
  "plantuml.includepaths": [
    "docs/diagrams"
  ],
  
  // 字符编码
  "plantuml.commandArgs": [
    "-charset",
    "UTF-8"
  ]
}
```

## 实时预览

插件支持实时预览：
1. 打开 `.puml` 文件
2. 按 `Alt+D` 打开预览
3. 编辑文件，预览会自动更新

## 常见问题

### Q: 预览显示乱码
A: 在设置中添加 UTF-8 编码支持：
```json
{
  "plantuml.commandArgs": ["-charset", "UTF-8"]
}
```

### Q: 无法预览，显示错误
A: 检查网络连接，或切换到本地渲染模式

### Q: 导出的图片质量不好
A: 尝试导出为 SVG 格式，或增加 DPI：
```json
{
  "plantuml.exportFormat": "svg"
}
```

### Q: 本地渲染很慢
A: 切换回在线服务器模式，或优化图表复杂度

## 生成项目所有图表

1. 打开 VS Code 工作区（项目根目录）
2. 按 `Cmd+Shift+P`
3. 输入 "PlantUML: Export Workspace Diagrams"
4. 所有 `.puml` 文件会被导出到对应的 `output` 目录

## 进阶技巧

### 1. 使用变量和常量

```plantuml
!define PRIMARY_COLOR #4CAF50
!define SECONDARY_COLOR #2196F3

rectangle "组件" PRIMARY_COLOR
```

### 2. 包含外部文件

```plantuml
!include common-styles.puml
!include components.puml
```

### 3. 使用预处理指令

```plantuml
!if %feature("chat")
  [Chat Service]
!endif
```

### 4. 自定义主题

```plantuml
!theme cerulean
' 或
!theme aws-orange
```

## 推荐工作流

1. **编辑**: 在 VS Code 中编辑 `.puml` 文件
2. **预览**: 按 `Alt+D` 实时预览
3. **调整**: 根据预览效果调整布局
4. **导出**: 右键导出为 PNG/SVG
5. **提交**: 将 `.puml` 源文件和生成的图片一起提交

## 更多资源

- [VS Code PlantUML 插件文档](https://github.com/qjebbs/vscode-plantuml)
- [PlantUML 官方文档](https://plantuml.com/zh/)
- [PlantUML 速查表](https://plantuml.com/zh/guide)

---

**提示**: 推荐使用在线服务器模式，无需安装任何依赖即可使用！

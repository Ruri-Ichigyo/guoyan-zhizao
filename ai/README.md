# Fruit Detection Service

轻量级的 FastAPI 服务，用于加载基于 DenseNet121 的果蔬分类模型并提供图片推理接口。

## ✨ 特性

- 🚀 高性能推理服务，支持 GPU 加速
- 🔄 模型缓存机制，优化加载性能
- 📊 完整的质量评估系统（颜色均匀度、表面缺陷、新鲜度分析）
- 🛡️ 完善的错误处理和日志记录
- 📝 类型安全，符合现代 Python 最佳实践
- ✅ 全面的单元测试覆盖

## 项目结构

```
.
├── app/                     # 核心应用代码
│   ├── api.py               # FastAPI 路由与应用初始化
│   ├── config.py            # 配置管理（环境变量支持）
│   ├── exceptions.py        # 自定义异常类
│   ├── logging_config.py    # 结构化日志配置
│   ├── model_cache.py       # 模型缓存系统（支持并发加载）
│   ├── model_metadata.py    # 模型元数据管理
│   ├── models.py            # 模型定义、注册与推理
│   ├── schemas.py           # Pydantic v2 数据模型
│   ├── service.py           # 核心推理服务逻辑
│   └── utils/               # 工具函数
│       ├── preprocess.py    # 图像预处理（统一接口）
│       └── postprocess.py   # 后处理与质量评估
├── model/                   # 模型文件目录
│   ├── fruit_densenet_weights.pth  # DenseNet121 权重
│   └── labels.json          # 类别标签（自动生成）
├── Test/                    # 测试数据集
├── tests/                   # 单元测试
│   └── test_refactored_components.py
├── main.py                  # ASGI 入口
├── pyproject.toml           # 项目配置（uv 管理）
└── uv.lock                  # 依赖锁文件
```

> 📝 **重构说明**: 代码已经过全面重构优化，消除了循环依赖、代码重复，并符合 Pydantic v2 和 Python 3.12+ 标准。

## 快速开始

### 环境要求

- Python 3.12+
- CUDA 支持（可选，用于 GPU 加速）
- uv 包管理器（推荐）或 pip
- Docker（可选，用于容器化部署）

### 使用 uv (推荐)

```bash
# 1. 安装 uv (如果还未安装)
curl -LsSf https://astral.sh/uv/install.sh | sh

# 2. 安装项目依赖
uv sync

# 3. 启动服务（方式一：使用项目脚本）
uv run fruit-detection

# 3. 启动服务（方式二：直接使用 uvicorn）
uv run uvicorn main:app --host 0.0.0.0 --port 8002 --reload
```

### 使用 Docker（推荐用于生产环境）

Docker构建已集成到项目主docker-compose配置中。如需独立运行AI服务：

```bash
# 1. 构建镜像
docker build -t fruit-detection:latest .

# 2. 运行容器
docker run -d \
  --name fruit-detection \
  -p 8002:8002 \
  -v $(pwd)/model:/app/model \
  fruit-detection:latest

# 3. 查看日志
docker logs -f fruit-detection
```

**重要说明**：
- 模型会在首次启动时自动初始化，不再需要下载ImageNet预训练权重
- 使用自定义训练权重文件 `model/fruit_densenet_weights.pth`
- 类别标签从 `Test/` 目录自动加载（60个水果/蔬菜类别）

### 使用传统 pip

```bash
# 1. 创建虚拟环境
python -m venv .venv
source .venv/bin/activate  # Linux/Mac
# .venv\Scripts\activate   # Windows

# 2. 安装依赖
pip install -r requirements.txt

# 3. 启动服务
python main.py
# 或
uvicorn main:app --host 0.0.0.0 --port 8002
```

### 环境变量配置

可以通过环境变量自定义服务配置：

```bash
# API 配置
export API_HOST=0.0.0.0
export API_PORT=8002

# 模型配置
export MODEL_PATH=model/fruit_densenet_weights.pth
export LABELS_PATH=model/labels.json
export DEFAULT_MODEL_NAME=densenet121-fruit-v1

# 推理配置
export USE_GPU=True
export TOP_K=3
export IMG_SIZE=224,224

# 调试模式
export DEBUG=False
```

### 开发与测试

```bash
# 运行全部测试
uv run pytest

# 运行特定测试
uv run pytest tests/test_refactored_components.py::TestConfig -v

# 运行测试并生成覆盖率报告
uv run pytest --cov=app --cov-report=html

# 代码格式化
uv run black app/ tests/

# 代码检查
uv run ruff check app/ tests/

# 类型检查
uv run mypy app/
```

## API 接口

### 健康检查

```bash
GET /health
```

返回服务状态：
```json
{
  "status": "ok"
}
```

### 推理接口

```bash
POST /infer
Content-Type: multipart/form-data

参数:
- upload_id: 上传任务ID (必填)
- file: 图片文件 (必填)
- model_name: 模型名称 (可选，默认使用配置中的默认模型)
```

返回示例：
```json
{
  "upload_id": "test-001",
  "status": "completed",
  "total_fruits": 1,
  "timestamp": "2025-10-13T12:00:00Z",
  "results": [
    {
      "class_name": "Apple Red 1",
      "confidence": 0.95,
      "bbox": [0.0, 0.0, 224.0, 224.0],
      "quality_grade": "A"
    }
  ],
  "metadata": {
    "model": "densenet121-fruit-v1",
    "model_version": "v1.0.0",
    "device": "cuda:0",
    "top_k": "3"
  }
}
```

### 模型管理

```bash
# 列出所有模型
GET /models

# 获取特定模型信息
GET /models/{model_name}

# 获取服务指标
GET /metrics
```

## 质量评估系统

服务提供完整的水果质量评估功能：

- **A级**: 优质水果（置信度 ≥ 0.9，表面无瑕疵，颜色均匀）
- **B级**: 良好水果（置信度 ≥ 0.7，表面轻微瑕疵）
- **C级**: 一般水果（置信度 < 0.7 或表面有明显缺陷）

评估维度：
- 颜色均匀度分析
- 表面缺陷检测
- 尺寸评分
- 新鲜度分析（支持水果特定分析：苹果、香蕉、柑橘等）

## 架构特点

### 模型缓存系统

- 自动缓存已加载的模型，避免重复加载
- 支持并发加载去重
- LRU 淘汰策略（最大缓存 10 个模型）

### 配置管理

- 基于 dataclass 的类型安全配置
- 支持环境变量覆盖
- 默认值完备，开箱即用

### 异常处理

自定义异常层次：
- `FruitDetectionError` - 基础异常
- `ModelLoadError` - 模型加载失败
- `ImageProcessingError` - 图像处理失败
- `InferenceError` - 推理失败
- `QualityAssessmentError` - 质量评估失败

### 日志系统

- 结构化日志输出
- 支持文件和控制台输出
- 可配置日志级别（DEBUG/INFO）

## 性能优化

1. **预处理统一**: 使用统一的预处理接口，避免代码重复
2. **Logger 优化**: 使用参数化格式，避免不必要的字符串格式化
3. **模型缓存**: 减少模型重复加载时间
4. **类型检查**: 编译时类型检查，减少运行时错误

## 测试覆盖

- ✅ 配置加载测试
- ✅ 模型初始化测试
- ✅ 图像预处理测试
- ✅ 质量评估测试
- ✅ 异常处理测试
- ✅ 元数据管理测试

## 故障排查

### 模型加载失败

检查：
1. 模型文件路径是否正确
2. Test/ 或 Training/ 目录是否存在且包含类别文件夹
3. labels.json 是否存在且格式正确

### GPU 不可用

如果 CUDA 不可用，服务会自动降级到 CPU 模式。检查：
```bash
python -c "import torch; print(torch.cuda.is_available())"
```

### 推理超时

调整：
- 减小 TOP_K 值
- 使用更小的 IMG_SIZE
- 确保 GPU 正常工作

## 贡献指南

1. Fork 本仓库
2. 创建功能分支
3. 编写测试用例
4. 确保测试通过
5. 提交 Pull Request

## 许可证

详见项目许可证文件。

## 更新日志

### v0.2.1 (2025-10-13)

**部署优化**:
- ✅ 修复模型加载问题，移除ImageNet预训练权重依赖
- ✅ 优化启动速度，直接使用自定义权重文件
- ✅ 改进Docker健康检查配置
- ✅ 更新文档，补充部署说明

### v0.2.0 (2025-10-13)

**重构优化**:
- ✅ 修复循环依赖问题（使用 TYPE_CHECKING）
- ✅ 统一预处理接口，消除代码重复
- ✅ 优化 logger 格式化性能
- ✅ 修复 dataclass 可变默认值问题
- ✅ 迁移到 Pydantic v2 标准
- ✅ 改进类型注解和代码质量
- ✅ 全面的测试覆盖

**功能改进**:
- 新增模型元数据管理
- 改进质量评估算法
- 优化错误处理机制

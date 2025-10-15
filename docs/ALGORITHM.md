# 算法说明与模型训练指南

## 概览

当前版本的果蔬智能分拣系统采用 **DenseNet121** 作为核心特征提取与分类模型，通过对整幅果蔬图像进行快速分类，结合传统视觉指标完成质量评估与等级判定。与早期的 YOLO 检测方案相比，DenseNet 提供了更轻量的推理路径，同时保持对类别与外观差异的敏感性。

整体推理流程如下所示（详见 `docs/PROJECT_GUIDE.md` 与 API 文档）：

```
客户端上传 → Rust 后端持久化 → FastAPI 推理服务
                                 ↓
                 预处理 → DenseNet → Top-K 结果
                                 ↓
                        质量评估 → 结构化响应
```

## 核心组件

### 1. DenseNet121 分类器

- **输入尺寸**：`3 × 224 × 224`
- **预训练权重**：基于 ImageNet 初始化
- **自定义分类头**：`Linear(num_features, num_classes)`，其中 `num_classes` 与 `model/labels.json` 中的类别数一致
- **输出形式**：对每个类别的 logits，推理阶段取 `Top-K (K ≤ 3)` 置信度结果

### 2. 图像预处理

预处理逻辑见 `ai/utils/preprocess.py`，与推理服务共享：

1. BGR → RGB 颜色空间转换  
2. `Resize(224, 224)`  
3. `ToTensor()`  
4. `Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225])`

该流程与 PyTorch `torchvision` 提供的 ImageNet 预训练规范保持一致。

### 3. 质量评估模块

位于 `ai/utils/postprocess.py`，通过以下指标对整幅图像进行打分：

- **颜色均匀度**：HSV 通道标准差  
- **表面缺陷**：边缘密度及纹理复杂度  
- **尺寸估计**：依据图像中果蔬所占比例推断  
- **新鲜度分析**：LAB 颜色空间亮度与色彩活跃度

最终得分映射为质量等级 `A / B / C`，并与分类置信度共同组成响应元数据。

### 4. 推理调用链路

- **Rust 后端**：接收前端上传的图片（二进制存入 PostgreSQL）并调用 AI 服务  
- **AI 服务**：FastAPI 提供 `POST /infer` 接口，直接接收文件字节并返回分类结果  
- **缓存层**：后端将推理结果写入 Redis，提升结果查询性能  
- **结构化响应**：包含 `model_version = densenet121-fruit-v1`、`top_k` 及质量等级指标

## 数据集准备

### 1. 数据组织方式

推理模型采用 **图像分类** 任务格式，可直接使用 `torchvision.datasets.ImageFolder`。推荐的目录结构如下：

```
dataset/
├── train/
│   ├── apple/
│   │   ├── img_001.jpg
│   │   └── ...
│   ├── banana/
│   └── ...
└── val/
    ├── apple/
    └── ...
```

> 若需要扩展到更多类别，只需在 `model/labels.json` 中新增映射，并确保训练数据同步。

### 2. 标签文件

`ai/model/labels.json` 以 `{ "index": "名称" }` 形式存储类别信息。推理服务初始化时会将键转换为整数，确保与模型输出的索引一致。

示例：

```json
{
  "0": "苹果",
  "1": "香蕉",
  "2": "橙子",
  "3": "葡萄",
  "4": "草莓"
}
```

### 3. 数据增强建议

- 水平翻转、轻微旋转（±15°）  
- 色彩抖动（亮度/对比度 ±0.1）  
- 随机裁剪与缩放（保持主要主体完整）

## 模型训练流程

基础训练脚本位于 `ai/train.py`，核心步骤包括：

```python
from torchvision import models, transforms
from torch.utils.data import DataLoader
import torch.nn as nn

model = models.densenet121(weights=models.DenseNet121_Weights.IMAGENET1K_V1)

# 冻结特征提取层以加速收敛
for param in model.features.parameters():
    param.requires_grad = False

# 重置分类头
num_features = model.classifier.in_features
model.classifier = nn.Linear(num_features, num_classes)
```

训练参数建议：

- **批大小**：128（根据显存调整）  
- **优化器**：Adam，`lr=1e-3`  
- **损失函数**：`CrossEntropyLoss`  
- **训练轮次**：10~20 次即可得到稳定收敛  
- **验证集**：每轮评估整体准确率，并保存最佳模型

训练完成后，请将权重保存至：

```
ai/model/fruit_densenet_weights.pth
```

推理服务会自动在该路径查找权重；若文件缺失，则退回至 ImageNet 预训练参数，并提示准确性受限。

## 推理服务实现概览

核心逻辑位于 `ai/main.py`：

1. 读取 `labels.json` 并计算 `num_classes` 与 `top_k`。  
2. 初始化 DenseNet121，加载（可选）自定义权重。  
3. FastAPI 端点接收 `multipart/form-data` 图像字节并解码为 `numpy.ndarray`。  
4. 预处理 → 模型推理 → `torch.softmax` 获取概率 → Top-K 取样。  
5. 计算质量等级，并以统一的 `InferenceResponse` 返回（参见 `docs/API.md`）。

与后端交互、缓存策略与表结构详见 `docs/PROJECT_GUIDE.md`。

## 质量评分细节

质量评分函数 `calculate_quality_grade` 结合多维特征进行综合判断：

1. **颜色均匀度**：基于 HSV 方差，越均匀得分越高  
2. **表面缺陷**：使用 Canny 边缘密度衡量纹理异常  
3. **尺寸指标**：估算果蔬在图像中的投影面积  
4. **新鲜度**：LAB 空间亮度与色彩活力  
5. **置信度融合**：模型输出置信度占 30% 权重

最终得分规则：

- `score ≥ 0.8` → `A`
- `0.6 ≤ score < 0.8` → `B`
- `score < 0.6` → `C`

遇到异常情况（如图像为空）时，自动降级为 `C`。

## 评估指标

推荐同时跟踪以下指标来评估模型表现：

- **Top-1 / Top-3 准确率**：衡量分类性能  
- **平均响应时间**：端到端推理延迟  
- **Kafka 处理吞吐量**：每分钟完成的任务数  
- **质量等级一致性**：与人工标注的差异度

## 常见问题

1. **提示未找到自定义权重**  
   - 确认 `ai/model/fruit_densenet_weights.pth` 是否存在  
   - 检查权重导出时的类别数量是否与 `labels.json` 一致

2. **类别名称异常**  
   - `labels.json` 的键必须是可转换为整数的字符串  
   - 确保索引从 0 开始连续编号

3. **置信度过低**  
   - 检查训练数据质量与类别均衡  
   - 适度解冻 DenseNet 部分特征层进行微调

## 术语与缩写

- **DenseNet**：密集连接卷积网络 (Densely Connected Convolutional Network)  
- **Top-K**：前 K 个置信度最高的预测  
- **ImageNet**：包含 1000 类的视觉预训练数据集  
- **CLAHE**：对比度受限自适应直方图均衡 (Contrast Limited Adaptive Histogram Equalization)

---

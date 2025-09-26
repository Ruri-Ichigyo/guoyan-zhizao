# 算法说明与模型训练指南

## 概览

果蔬智能分拣系统采用基于深度学习的目标检测技术，结合计算机视觉算法进行果蔬识别、质量评估和缺陷检测。本文档详细介绍了算法原理、模型训练流程和微调方法。

## 算法架构

### 1. 整体框架

```
输入图像 → 预处理 → 目标检测 → 特征提取 → 质量评估 → 分拣决策
    ↓         ↓        ↓        ↓        ↓        ↓
  图像增强   归一化   YOLOv8   ResNet   质量分级  控制信号
```

### 2. 核心组件

#### 2.1 目标检测模型
- **基础架构**: YOLOv8 (You Only Look Once v8)
- **骨干网络**: CSPNet + Focus
- **颈部网络**: PAN (Path Aggregation Network)
- **检测头**: Anchor-free detection head

#### 2.2 质量评估模型
- **颜色分析**: HSV 色彩空间特征提取
- **形状分析**: 轮廓检测与几何特征
- **表面缺陷检测**: 边缘检测 + 纹理分析
- **新鲜度评估**: 色泽均匀性 + 饱和度分析

#### 2.3 分拣决策系统
- **多因子加权**: 置信度 + 质量分级 + 尺寸
- **阈值优化**: 自适应阈值调整
- **实时控制**: PID 控制算法

## 数据集准备

### 1. 数据收集

#### 数据来源
- 真实生产线采集: 70%
- 实验室标准拍摄: 20%
- 公开数据集补充: 10%

#### 图像要求
```python
# 图像规格
IMAGE_SIZE = (640, 640)
CHANNELS = 3  # RGB
FORMAT = 'jpg' or 'png'
QUALITY = 'high'  # > 300 DPI

# 拍摄条件
LIGHTING = 'uniform'  # 均匀光照
BACKGROUND = 'neutral'  # 中性背景
ANGLE = 'multiple'  # 多角度拍摄
```

### 2. 数据标注

#### 标注格式 (YOLO)
```
# labels/image_001.txt
class_id center_x center_y width height
0 0.5 0.3 0.2 0.4
1 0.7 0.6 0.15 0.25
```

#### 类别定义
```json
{
  "classes": {
    "0": "苹果",
    "1": "香蕉",
    "2": "橙子",
    "3": "葡萄",
    "4": "草莓",
    "5": "菠萝",
    "6": "西瓜",
    "7": "桃子",
    "8": "梨",
    "9": "柠檬",
    "10": "樱桃",
    "11": "芒果",
    "12": "奇异果",
    "13": "石榴",
    "14": "椰子",
    "15": "火龙果"
  }
}
```

#### 质量标注
```json
{
  "quality_attributes": {
    "grade": ["A", "B", "C"],
    "defects": {
      "surface_scratch": "boolean",
      "color_uneven": "boolean",
      "shape_abnormal": "boolean",
      "rot_spots": "boolean"
    },
    "freshness": "float[0-1]",
    "ripeness": "float[0-1]"
  }
}
```

### 3. 数据集结构

```
dataset/
├── images/
│   ├── train/
│   │   ├── apple_001.jpg
│   │   ├── apple_002.jpg
│   │   └── ...
│   ├── val/
│   │   ├── apple_val_001.jpg
│   │   └── ...
│   └── test/
│       ├── apple_test_001.jpg
│       └── ...
├── labels/
│   ├── train/
│   │   ├── apple_001.txt
│   │   └── ...
│   ├── val/
│   └── test/
├── dataset.yaml
└── classes.txt
```

### 4. 数据增强

```python
# 数据增强配置
augmentation_config = {
    'mosaic': 0.5,              # 马赛克增强
    'mixup': 0.3,               # 混合增强
    'copy_paste': 0.3,          # 复制粘贴
    'hsv_h': 0.015,             # 色调调整
    'hsv_s': 0.7,               # 饱和度调整
    'hsv_v': 0.4,               # 亮度调整
    'degrees': 10.0,            # 旋转角度
    'translate': 0.1,           # 平移比例
    'scale': 0.5,               # 缩放比例
    'shear': 2.0,               # 剪切变换
    'perspective': 0.0001,      # 透视变换
    'flipud': 0.0,              # 上下翻转
    'fliplr': 0.5,              # 左右翻转
}
```

## 模型训练

### 1. 环境配置

```bash
# 创建虚拟环境
conda create -n fruit-detection python=3.9
conda activate fruit-detection

# 安装依赖
pip install ultralytics>=8.0.0
pip install torch torchvision torchaudio
pip install opencv-python
pip install albumentations
pip install wandb  # 可选：训练监控
```

### 2. 训练配置

#### dataset.yaml
```yaml
# 数据集配置
path: /path/to/fruit-dataset
train: images/train
val: images/val
test: images/test

# 类别数量
nc: 16

# 类别名称
names:
  0: 苹果
  1: 香蕉
  2: 橙子
  3: 葡萄
  4: 草莓
  5: 菠萝
  6: 西瓜
  7: 桃子
  8: 梨
  9: 柠檬
  10: 樱桃
  11: 芒果
  12: 奇异果
  13: 石榴
  14: 椰子
  15: 火龙果
```

#### 训练超参数
```python
# training_config.py
training_params = {
    'epochs': 300,
    'batch_size': 16,
    'imgsz': 640,
    'lr0': 0.01,
    'lrf': 0.01,
    'momentum': 0.937,
    'weight_decay': 0.0005,
    'warmup_epochs': 3.0,
    'warmup_momentum': 0.8,
    'warmup_bias_lr': 0.1,
    'box': 7.5,
    'cls': 0.5,
    'dfl': 1.5,
    'fl_gamma': 0.0,
    'label_smoothing': 0.0,
    'nbs': 64,
    'hsv_h': 0.015,
    'hsv_s': 0.7,
    'hsv_v': 0.4,
    'degrees': 0.0,
    'translate': 0.1,
    'scale': 0.5,
    'shear': 0.0,
    'perspective': 0.0,
    'flipud': 0.0,
    'fliplr': 0.5,
    'mosaic': 1.0,
    'mixup': 0.0,
    'copy_paste': 0.0,
}
```

### 3. 训练脚本

```python
# train.py
from ultralytics import YOLO
import torch
import wandb

def train_fruit_detection_model():
    # 初始化 Weights & Biases (可选)
    wandb.init(project="fruit-detection", name="yolov8n-fruit-v1")

    # 加载预训练模型
    model = YOLO('yolov8n.pt')  # 或 yolov8s.pt, yolov8m.pt, yolov8l.pt, yolov8x.pt

    # 开始训练
    results = model.train(
        data='dataset.yaml',
        epochs=300,
        imgsz=640,
        batch=16,
        device='0',  # GPU 设备
        workers=8,
        project='fruit-detection',
        name='yolov8n-fruit-v1',
        exist_ok=True,
        pretrained=True,
        optimizer='AdamW',
        verbose=True,
        seed=42,
        deterministic=True,
        single_cls=False,
        rect=False,
        cos_lr=False,
        close_mosaic=10,
        resume=False,
        amp=True,  # 自动混合精度
        fraction=1.0,
        profile=False,
        # 回调函数
        callbacks={
            'on_pretrain_routine_end': lambda trainer: print("预训练结束"),
            'on_train_epoch_end': lambda trainer: wandb.log({
                'epoch': trainer.epoch,
                'loss': trainer.loss,
                'lr': trainer.optimizer.param_groups[0]['lr']
            }) if wandb.run else None,
        }
    )

    return results

if __name__ == "__main__":
    # 设置设备
    device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    print(f"Using device: {device}")

    # 开始训练
    results = train_fruit_detection_model()
    print("训练完成！")
```

### 4. 多GPU训练

```python
# multi_gpu_train.py
import torch
import torch.distributed as dist
from ultralytics import YOLO

def setup_distributed():
    dist.init_process_group("nccl")
    torch.cuda.set_device(int(os.environ["LOCAL_RANK"]))

def train_distributed():
    setup_distributed()

    model = YOLO('yolov8n.pt')

    # 分布式训练
    results = model.train(
        data='dataset.yaml',
        epochs=300,
        imgsz=640,
        batch=32,  # 总 batch size
        device=[0, 1, 2, 3],  # 多GPU
        workers=16,
        project='fruit-detection-distributed',
        name='yolov8n-fruit-multi-gpu',
    )

    return results

# 启动命令
# torchrun --nproc_per_node=4 multi_gpu_train.py
```

## 模型微调

### 1. 领域自适应

```python
# domain_adaptation.py
def fine_tune_for_specific_fruits(base_model_path, specific_dataset):
    # 加载预训练模型
    model = YOLO(base_model_path)

    # 冻结主干网络
    for param in model.model[:10].parameters():
        param.requires_grad = False

    # 微调训练
    results = model.train(
        data=specific_dataset,
        epochs=100,
        imgsz=640,
        batch=8,
        lr0=0.001,  # 较小的学习率
        warmup_epochs=5,
        project='fine-tuning',
        name='apple-specialist',
    )

    return results

# 苹果专门化模型
apple_results = fine_tune_for_specific_fruits(
    'yolov8n-fruit.pt',
    'apple_dataset.yaml'
)
```

### 2. 渐进式学习

```python
# progressive_learning.py
class ProgressiveLearning:
    def __init__(self, base_model):
        self.model = YOLO(base_model)
        self.stages = [
            {'classes': ['苹果', '橙子'], 'epochs': 50},
            {'classes': ['苹果', '橙子', '香蕉', '葡萄'], 'epochs': 30},
            {'classes': 'all', 'epochs': 20},
        ]

    def train_progressive(self):
        for i, stage in enumerate(self.stages):
            print(f"Training Stage {i+1}: {stage['classes']}")

            # 准备数据集
            dataset_config = self.prepare_stage_dataset(stage['classes'])

            # 训练当前阶段
            results = self.model.train(
                data=dataset_config,
                epochs=stage['epochs'],
                project=f'progressive-stage-{i+1}',
                name=f'stage-{i+1}',
                resume=i > 0,  # 从之前阶段继续
            )

            # 保存阶段模型
            self.model.save(f'stage_{i+1}_model.pt')

    def prepare_stage_dataset(self, classes):
        # 根据类别准备数据集配置
        pass
```

### 3. 知识蒸馏

```python
# knowledge_distillation.py
import torch.nn.functional as F

class KnowledgeDistillation:
    def __init__(self, teacher_model, student_model, temperature=4.0, alpha=0.3):
        self.teacher = YOLO(teacher_model)
        self.student = YOLO(student_model)
        self.temperature = temperature
        self.alpha = alpha

    def distillation_loss(self, student_logits, teacher_logits, labels):
        # 蒸馏损失
        kd_loss = F.kl_div(
            F.log_softmax(student_logits / self.temperature, dim=1),
            F.softmax(teacher_logits / self.temperature, dim=1),
            reduction='batchmean'
        ) * (self.temperature ** 2)

        # 标准损失
        ce_loss = F.cross_entropy(student_logits, labels)

        # 组合损失
        total_loss = self.alpha * kd_loss + (1 - self.alpha) * ce_loss
        return total_loss

    def train_student(self, dataset, epochs=200):
        # 教师模型预测模式
        self.teacher.eval()

        # 学生模型训练
        results = self.student.train(
            data=dataset,
            epochs=epochs,
            loss_fn=self.distillation_loss,
            project='knowledge-distillation',
            name='student-model',
        )

        return results
```

## 质量评估算法

### 1. 颜色分析

```python
# quality_assessment.py
import cv2
import numpy as np

class QualityAssessment:
    def __init__(self):
        self.color_standards = {
            '苹果': {'h_range': (0, 15), 's_min': 100, 'v_min': 100},
            '橙子': {'h_range': (10, 25), 's_min': 150, 'v_min': 120},
            '香蕉': {'h_range': (20, 30), 's_min': 100, 'v_min': 150},
        }

    def analyze_color_quality(self, image, fruit_class):
        # 转换到 HSV 色彩空间
        hsv = cv2.cvtColor(image, cv2.COLOR_BGR2HSV)

        # 获取颜色标准
        standard = self.color_standards.get(fruit_class, {})

        # 颜色均匀性分析
        uniformity = self.calculate_color_uniformity(hsv)

        # 饱和度分析
        saturation_score = self.analyze_saturation(hsv, standard)

        # 亮度分析
        brightness_score = self.analyze_brightness(hsv, standard)

        # 综合评分
        color_score = (uniformity + saturation_score + brightness_score) / 3

        return {
            'color_score': color_score,
            'uniformity': uniformity,
            'saturation': saturation_score,
            'brightness': brightness_score
        }

    def calculate_color_uniformity(self, hsv_image):
        h, s, v = cv2.split(hsv_image)

        # 计算色调标准差
        h_std = np.std(h)
        s_std = np.std(s)

        # 归一化得分 (标准差越小，均匀性越好)
        uniformity = max(0, 1 - (h_std + s_std) / 255)

        return uniformity
```

### 2. 形状分析

```python
def analyze_shape_quality(self, image, fruit_class):
    # 二值化处理
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
    _, binary = cv2.threshold(gray, 127, 255, cv2.THRESH_BINARY)

    # 轮廓检测
    contours, _ = cv2.findContours(binary, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

    if not contours:
        return {'shape_score': 0}

    # 最大轮廓
    main_contour = max(contours, key=cv2.contourArea)

    # 形状特征
    area = cv2.contourArea(main_contour)
    perimeter = cv2.arcLength(main_contour, True)

    # 圆形度 (4π × 面积 / 周长²)
    circularity = 4 * np.pi * area / (perimeter ** 2) if perimeter > 0 else 0

    # 长宽比
    rect = cv2.minAreaRect(main_contour)
    width, height = rect[1]
    aspect_ratio = max(width, height) / min(width, height) if min(width, height) > 0 else 0

    # 形状标准
    shape_standards = {
        '苹果': {'circularity': (0.7, 1.0), 'aspect_ratio': (1.0, 1.3)},
        '橙子': {'circularity': (0.8, 1.0), 'aspect_ratio': (1.0, 1.2)},
        '香蕉': {'circularity': (0.2, 0.5), 'aspect_ratio': (3.0, 5.0)},
    }

    standard = shape_standards.get(fruit_class, {})

    # 评分计算
    shape_score = self.calculate_shape_score(
        circularity, aspect_ratio, standard
    )

    return {
        'shape_score': shape_score,
        'circularity': circularity,
        'aspect_ratio': aspect_ratio,
        'area': area
    }
```

### 3. 表面缺陷检测

```python
def detect_surface_defects(self, image):
    # 高斯模糊去噪
    blurred = cv2.GaussianBlur(image, (5, 5), 0)

    # 边缘检测
    edges = cv2.Canny(blurred, 50, 150)

    # 形态学操作
    kernel = np.ones((3, 3), np.uint8)
    edges = cv2.morphologyEx(edges, cv2.MORPH_CLOSE, kernel)

    # 缺陷区域检测
    contours, _ = cv2.findContours(edges, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

    defects = []
    for contour in contours:
        area = cv2.contourArea(contour)
        if area > 50:  # 过滤小噪点
            # 缺陷特征
            rect = cv2.boundingRect(contour)
            aspect_ratio = rect[2] / rect[3]

            # 缺陷分类
            defect_type = self.classify_defect(contour, area, aspect_ratio)

            defects.append({
                'type': defect_type,
                'area': area,
                'bbox': rect,
                'severity': self.calculate_defect_severity(area, aspect_ratio)
            })

    return defects

def classify_defect(self, contour, area, aspect_ratio):
    # 基于形状特征的缺陷分类
    if aspect_ratio > 3:
        return '表面划痕'
    elif area > 500:
        return '腐烂斑点'
    elif aspect_ratio < 0.5:
        return '虫蛀孔洞'
    else:
        return '表面损伤'
```

## 模型评估

### 1. 评估指标

```python
# evaluation.py
from ultralytics import YOLO
import numpy as np
from sklearn.metrics import classification_report, confusion_matrix

def evaluate_model(model_path, test_dataset):
    model = YOLO(model_path)

    # 推理评估
    results = model.val(
        data=test_dataset,
        split='test',
        imgsz=640,
        batch=16,
        conf=0.25,
        iou=0.7,
        device='0',
        project='evaluation',
        name='test-results',
    )

    # 提取评估指标
    metrics = {
        'mAP@0.5': results.box.map50,
        'mAP@0.5:0.95': results.box.map,
        'Precision': results.box.mp,
        'Recall': results.box.mr,
        'F1-Score': 2 * (results.box.mp * results.box.mr) / (results.box.mp + results.box.mr)
    }

    return metrics, results

def detailed_evaluation(model_path, test_data):
    model = YOLO(model_path)

    # 按类别评估
    class_metrics = {}
    all_predictions = []
    all_labels = []

    for image_path, label_path in test_data:
        # 模型预测
        results = model.predict(image_path, conf=0.25, iou=0.7)

        # 提取预测结果
        predictions = extract_predictions(results)
        labels = load_labels(label_path)

        all_predictions.extend(predictions)
        all_labels.extend(labels)

    # 分类报告
    report = classification_report(all_labels, all_predictions, output_dict=True)

    # 混淆矩阵
    cm = confusion_matrix(all_labels, all_predictions)

    return report, cm
```

### 2. 可视化分析

```python
# visualization.py
import matplotlib.pyplot as plt
import seaborn as sns

def plot_training_curves(results_dir):
    # 加载训练结果
    results = torch.load(f"{results_dir}/results.pt")

    fig, axes = plt.subplots(2, 2, figsize=(15, 10))

    # 损失曲线
    axes[0, 0].plot(results['train/loss'], label='Train Loss')
    axes[0, 0].plot(results['val/loss'], label='Val Loss')
    axes[0, 0].set_title('Loss Curves')
    axes[0, 0].legend()

    # mAP 曲线
    axes[0, 1].plot(results['val/mAP@0.5'], label='mAP@0.5')
    axes[0, 1].plot(results['val/mAP@0.5:0.95'], label='mAP@0.5:0.95')
    axes[0, 1].set_title('mAP Curves')
    axes[0, 1].legend()

    # 精确率和召回率
    axes[1, 0].plot(results['val/precision'], label='Precision')
    axes[1, 0].plot(results['val/recall'], label='Recall')
    axes[1, 0].set_title('Precision & Recall')
    axes[1, 0].legend()

    # 学习率
    axes[1, 1].plot(results['lr'])
    axes[1, 1].set_title('Learning Rate')

    plt.tight_layout()
    plt.savefig('training_analysis.png', dpi=300, bbox_inches='tight')

def plot_confusion_matrix(cm, class_names):
    plt.figure(figsize=(12, 10))
    sns.heatmap(cm, annot=True, fmt='d', cmap='Blues',
                xticklabels=class_names, yticklabels=class_names)
    plt.title('Confusion Matrix')
    plt.xlabel('Predicted Label')
    plt.ylabel('True Label')
    plt.savefig('confusion_matrix.png', dpi=300, bbox_inches='tight')

def plot_detection_examples(model, test_images, save_dir):
    model = YOLO(model)

    for i, image_path in enumerate(test_images[:10]):
        results = model.predict(image_path, conf=0.25, save=False)

        # 绘制检测结果
        annotated = results[0].plot()

        plt.figure(figsize=(12, 8))
        plt.imshow(cv2.cvtColor(annotated, cv2.COLOR_BGR2RGB))
        plt.axis('off')
        plt.title(f'Detection Example {i+1}')
        plt.savefig(f'{save_dir}/detection_example_{i+1}.png',
                   dpi=300, bbox_inches='tight')
        plt.close()
```

## 模型优化

### 1. 模型压缩

```python
# model_compression.py
from ultralytics import YOLO

def compress_model(model_path):
    model = YOLO(model_path)

    # TensorRT 优化 (GPU)
    model.export(format='engine', device=0, workspace=4)

    # ONNX 优化 (跨平台)
    model.export(format='onnx', dynamic=True, simplify=True)

    # OpenVINO 优化 (Intel CPU)
    model.export(format='openvino', half=True)

    # CoreML 优化 (Apple 设备)
    model.export(format='coreml', int8=True)

def quantize_model(model_path, calibration_data):
    # INT8 量化
    model = YOLO(model_path)

    # 量化配置
    quantization_config = {
        'calibration_dataset': calibration_data,
        'num_calibration_batches': 100,
        'quantization_scheme': 'symmetric',
        'per_channel': True,
    }

    # 执行量化
    quantized_model = model.quantize(**quantization_config)
    quantized_model.save('quantized_model.pt')

    return quantized_model
```

### 2. 推理优化

```python
# inference_optimization.py
import tensorrt as trt
import pycuda.driver as cuda
import numpy as np

class TensorRTInference:
    def __init__(self, engine_path):
        self.engine_path = engine_path
        self.engine = None
        self.context = None
        self.inputs = []
        self.outputs = []
        self.bindings = []
        self.stream = None

        self.load_engine()

    def load_engine(self):
        TRT_LOGGER = trt.Logger(trt.Logger.WARNING)
        with open(self.engine_path, 'rb') as f, trt.Runtime(TRT_LOGGER) as runtime:
            self.engine = runtime.deserialize_cuda_engine(f.read())

        self.context = self.engine.create_execution_context()
        self.stream = cuda.Stream()

        # 分配 GPU 内存
        for binding in self.engine:
            size = trt.volume(self.engine.get_binding_shape(binding)) * self.engine.max_batch_size
            dtype = trt.nptype(self.engine.get_binding_dtype(binding))
            host_mem = cuda.pagelocked_empty(size, dtype)
            device_mem = cuda.mem_alloc(host_mem.nbytes)
            self.bindings.append(int(device_mem))

            if self.engine.binding_is_input(binding):
                self.inputs.append({'host': host_mem, 'device': device_mem})
            else:
                self.outputs.append({'host': host_mem, 'device': device_mem})

    def infer(self, input_data):
        # 复制输入数据到 GPU
        np.copyto(self.inputs[0]['host'], input_data.ravel())
        cuda.memcpy_htod_async(self.inputs[0]['device'], self.inputs[0]['host'], self.stream)

        # 执行推理
        self.context.execute_async_v2(bindings=self.bindings, stream_handle=self.stream.handle)

        # 复制结果到 CPU
        cuda.memcpy_dtoh_async(self.outputs[0]['host'], self.outputs[0]['device'], self.stream)
        self.stream.synchronize()

        return self.outputs[0]['host'].reshape((-1, 85))  # [batch, 85]
```

## 部署和监控

### 1. 模型版本管理

```python
# model_versioning.py
import mlflow
import mlflow.pytorch

def log_model_experiment(model, dataset, metrics, hyperparameters):
    with mlflow.start_run():
        # 记录超参数
        mlflow.log_params(hyperparameters)

        # 记录指标
        mlflow.log_metrics(metrics)

        # 记录模型
        mlflow.pytorch.log_model(model, "fruit-detection-model")

        # 记录数据集信息
        mlflow.log_param("dataset_size", len(dataset))
        mlflow.log_param("dataset_version", dataset.version)

        # 标记模型阶段
        model_version = mlflow.register_model(
            f"runs:/{mlflow.active_run().info.run_id}/fruit-detection-model",
            "FruitDetectionModel"
        )

        # 转换到生产阶段
        client = mlflow.tracking.MlflowClient()
        client.transition_model_version_stage(
            name="FruitDetectionModel",
            version=model_version.version,
            stage="Production"
        )

def load_production_model():
    # 加载生产环境模型
    model = mlflow.pytorch.load_model(
        model_uri="models:/FruitDetectionModel/Production"
    )
    return model
```

### 2. 性能监控

```python
# model_monitoring.py
import logging
import time
from collections import defaultdict, deque

class ModelMonitor:
    def __init__(self, window_size=1000):
        self.window_size = window_size
        self.inference_times = deque(maxlen=window_size)
        self.confidence_scores = deque(maxlen=window_size)
        self.class_distributions = defaultdict(int)
        self.error_count = 0
        self.total_requests = 0

        self.logger = logging.getLogger(__name__)

    def log_inference(self, inference_time, results):
        self.total_requests += 1
        self.inference_times.append(inference_time)

        if results:
            for result in results:
                confidence = result.get('confidence', 0)
                class_name = result.get('class_name', 'unknown')

                self.confidence_scores.append(confidence)
                self.class_distributions[class_name] += 1

                # 检查置信度异常
                if confidence < 0.5:
                    self.logger.warning(f"Low confidence detection: {confidence}")
        else:
            self.error_count += 1

    def get_metrics(self):
        if not self.inference_times:
            return {}

        return {
            'avg_inference_time': sum(self.inference_times) / len(self.inference_times),
            'max_inference_time': max(self.inference_times),
            'avg_confidence': sum(self.confidence_scores) / len(self.confidence_scores) if self.confidence_scores else 0,
            'error_rate': self.error_count / self.total_requests if self.total_requests > 0 else 0,
            'class_distribution': dict(self.class_distributions),
            'total_requests': self.total_requests
        }

    def check_model_drift(self, baseline_distribution):
        # 检查数据漂移
        current_distribution = dict(self.class_distributions)

        # 计算 KL 散度
        kl_divergence = self.calculate_kl_divergence(
            baseline_distribution, current_distribution
        )

        if kl_divergence > 0.1:  # 阈值
            self.logger.warning(f"Data drift detected: KL divergence = {kl_divergence}")
            return True

        return False

    def calculate_kl_divergence(self, p, q):
        # KL 散度计算
        kl_div = 0
        for class_name in set(p.keys()) | set(q.keys()):
            p_prob = p.get(class_name, 1e-10)
            q_prob = q.get(class_name, 1e-10)
            kl_div += p_prob * np.log(p_prob / q_prob)
        return kl_div
```

## 持续学习

### 1. 在线学习

```python
# online_learning.py
class OnlineLearningSystem:
    def __init__(self, base_model_path):
        self.model = YOLO(base_model_path)
        self.feedback_buffer = []
        self.retrain_threshold = 100
        self.performance_threshold = 0.9

    def collect_feedback(self, image, prediction, ground_truth, user_feedback):
        feedback_item = {
            'image': image,
            'prediction': prediction,
            'ground_truth': ground_truth,
            'user_feedback': user_feedback,
            'timestamp': time.time()
        }

        self.feedback_buffer.append(feedback_item)

        # 检查是否需要重新训练
        if len(self.feedback_buffer) >= self.retrain_threshold:
            self.trigger_retraining()

    def trigger_retraining(self):
        # 准备反馈数据
        feedback_dataset = self.prepare_feedback_dataset()

        # 增量学习
        self.incremental_learning(feedback_dataset)

        # 清空缓冲区
        self.feedback_buffer = []

    def incremental_learning(self, new_data):
        # 使用较小的学习率进行微调
        results = self.model.train(
            data=new_data,
            epochs=10,
            lr0=0.0001,
            batch=4,
            project='online-learning',
            name=f'update_{int(time.time())}',
            resume=True
        )

        # 验证新模型性能
        if self.validate_model_performance(results):
            self.deploy_updated_model()
        else:
            self.logger.warning("Updated model performance below threshold, reverting changes")
```

### 2. 主动学习

```python
# active_learning.py
class ActiveLearningSystem:
    def __init__(self, model_path, uncertainty_threshold=0.3):
        self.model = YOLO(model_path)
        self.uncertainty_threshold = uncertainty_threshold
        self.uncertain_samples = []

    def predict_with_uncertainty(self, image):
        # 多次推理获取不确定性
        predictions = []
        for _ in range(10):  # Monte Carlo Dropout
            result = self.model.predict(image, conf=0.1)
            predictions.append(result)

        # 计算预测方差
        uncertainty = self.calculate_prediction_variance(predictions)

        # 收集不确定样本
        if uncertainty > self.uncertainty_threshold:
            self.uncertain_samples.append({
                'image': image,
                'uncertainty': uncertainty,
                'predictions': predictions
            })

        return predictions[0], uncertainty

    def calculate_prediction_variance(self, predictions):
        # 计算预测置信度的方差
        confidences = []
        for pred in predictions:
            if pred.boxes:
                conf = [box.conf.item() for box in pred.boxes]
                confidences.extend(conf)

        return np.var(confidences) if confidences else 0

    def suggest_annotation_samples(self, num_samples=50):
        # 按不确定性排序
        sorted_samples = sorted(
            self.uncertain_samples,
            key=lambda x: x['uncertainty'],
            reverse=True
        )

        return sorted_samples[:num_samples]
```

通过以上详细的算法说明和训练指南，您可以从零开始构建和优化果蔬智能分拣系统的AI模型，实现高精度的目标检测和质量评估功能。
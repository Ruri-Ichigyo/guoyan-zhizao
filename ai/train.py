import os
from pathlib import Path

import pandas as pd
import torch
import torch.nn as nn
import torch.optim as optim
from PIL import Image
from torch.utils.data import DataLoader, Dataset
from torchvision import datasets, models, transforms

# 数据集路径（可通过环境变量覆盖）
BASE_DIR = Path(__file__).resolve().parent
data_dir = os.environ.get(
    'TRAIN_DATA_DIR',
    str(BASE_DIR / 'Fruit-Images-Dataset-master' / 'Training')
)
val_dir = os.environ.get(
    'VAL_DATA_DIR',
    str(BASE_DIR / 'Fruit-Images-Dataset-master' / 'test-multiple_fruits')
)
output_dir = Path(os.environ.get('OUTPUT_DIR', BASE_DIR / 'model'))
output_dir.mkdir(parents=True, exist_ok=True)

# 数据预处理
transform = transforms.Compose([
    transforms.Resize((224, 224)),  # 调整图片大小
    transforms.ToTensor(),  # 转换为Tensor
    transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225])  # 标准化
])


# 自定义无标签图片数据集
class UnlabeledImageDataset(Dataset):
    def __init__(self, folder, transform=None):
        self.paths = [os.path.join(folder, f) for f in os.listdir(folder)
                      if f.lower().endswith(('.jpg', '.jpeg', '.png'))]
        self.transform = transform

    def __len__(self):
        return len(self.paths)

    def __getitem__(self, idx):
        img_path = self.paths[idx]
        img = Image.open(img_path).convert('RGB')
        if self.transform:
            img = self.transform(img)
        return img, img_path  # 返回路径方便显示预测结果


# 加载数据集
train_data = datasets.ImageFolder(root=data_dir, transform=transform)

# DataLoader
batch_size = int(os.environ.get('BATCH_SIZE', 128))
train_loader = DataLoader(train_data, batch_size=batch_size, shuffle=True)

# 使用预训练的DenseNet模型
model = models.densenet121(weights=models.DenseNet121_Weights.IMAGENET1K_V1)  # 使用densenet121，可以根据需求替换为其他DenseNet模型
for param in model.features.parameters():
    param.requires_grad = False
num_features = model.classifier.in_features
model.classifier = nn.Linear(num_features, 131)  # 262是Fruit-262数据集的类别数量

# 使用GPU进行训练
device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
model = model.to(device)

# 损失函数和优化器
criterion = nn.CrossEntropyLoss()  # 多分类交叉熵损失函数
learning_rate = float(os.environ.get('LEARNING_RATE', 0.001))
optimizer = optim.Adam(model.parameters(), lr=learning_rate)  # Adam优化器


# 训练函数
def train(model, train_loader, criterion, optimizer, device, epochs=10):
    model.train()  # 设置模型为训练模式
    for epoch in range(epochs):
        running_loss = 0.0
        for inputs, labels in train_loader:
            inputs, labels = inputs.to(device), labels.to(device)

            optimizer.zero_grad()  # 清除上一步的梯度

            # 前向传播
            outputs = model(inputs)

            # 计算损失
            loss = criterion(outputs, labels)
            loss.backward()  # 反向传播
            optimizer.step()  # 更新参数

            running_loss += loss.item()

        print(f"Epoch [{epoch + 1}/{epochs}], Loss: {running_loss / len(train_loader)}")


# 验证函数
def validate(model, val_loader, criterion, device):
    model.eval()  # 设置模型为评估模式
    correct = 0
    total = 0
    with torch.no_grad():
        for inputs, labels in val_loader:
            inputs, labels = inputs.to(device), labels.to(device)

            # 前向传播
            outputs = model(inputs)
            _, predicted = torch.max(outputs, 1)

            total += labels.size(0)
            correct += (predicted == labels).sum().item()

    accuracy = 100 * correct / total
    print(f"Validation Accuracy: {accuracy:.2f}%")
    return accuracy


def predict_images(model, data_loader, device, class_names):
    model.eval()  # 设置为推理模式
    predictions = []

    with torch.no_grad():
        for imgs, paths in data_loader:
            imgs = imgs.to(device)

            outputs = model(imgs)
            _, predicted = torch.max(outputs, 1)

            for i in range(len(paths)):
                class_id = predicted[i].item()
                class_name = class_names[class_id]
                predictions.append({
                    'image_path': paths[i],
                    'predicted_label': class_name
                })
                print(f"图片: {os.path.basename(paths[i])} -> 预测类别: {class_name}")

    return predictions


# 训练并验证
epochs = int(os.environ.get('EPOCHS', 10))
train(model, train_loader, criterion, optimizer, device, epochs)
torch.save(model.state_dict(), output_dir / "fruit_densenet_weights.pth")
# 获取类别名称列表（从训练集中）
class_names = train_data.classes

# 加载无标签测试集
val_dataset = UnlabeledImageDataset(val_dir, transform=transform)
val_loader = DataLoader(val_dataset, batch_size=1, shuffle=False)

# 执行预测
results = predict_images(model, val_loader, device, class_names)



df = pd.DataFrame(results)
predictions_path = output_dir / "predictions.csv"
df.to_csv(predictions_path, index=False)
print(f"预测结果已保存到 {predictions_path}")

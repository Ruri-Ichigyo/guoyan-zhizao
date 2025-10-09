# Fruit DenseNet Weights

This placeholder represents the serialized DenseNet121 weights used by the AI inference service.
The actual `fruit_densenet_weights.pth` file should be produced during training and copied into
this directory before starting the service.

## Expected Format

- Architecture: `torchvision.models.densenet121`
- Classifier: `nn.Linear(in_features, num_classes)` where `num_classes` matches `labels.json`
- Serialization: `torch.save(model.state_dict(), "fruit_densenet_weights.pth")`

## Training Shortcut

```bash
cd ai
pip install -r requirements.txt
python train.py  # adjust dataset paths inside the script as needed
cp /kaggle/working/fruit_densenet_weights.pth model/fruit_densenet_weights.pth
```

Ensure that `labels.json` contains the same class ordering used during training, otherwise
predictions will be misaligned.

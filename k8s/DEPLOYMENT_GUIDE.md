# 果蔬智能分拣系统 Kubernetes 部署说明

## 概述

本文档提供了果蔬智能分拣系统完整的Kubernetes部署配置，包括所有必需的基础设施组件和应用服务。

## 系统架构

### 基础设施组件
- **PostgreSQL**: 主数据库，存储果蔬分类、检测结果和分拣任务
- **Redis**: 缓存服务，提高查询性能
- **Kafka**: 消息队列，处理图像事件和推理结果
- **MinIO**: 对象存储，管理图像文件和模型
- **Kong**: API网关，统一API入口和路由管理

### 应用服务
- **AI推理服务**: YOLOv8模型推理，图像识别
- **上传服务**: 处理图像上传和预处理
- **查询服务**: 提供数据查询和报告功能
- **分拣控制服务**: 控制分拣设备和流程
- **前端服务**: Web用户界面

### 监控和运维
- **Prometheus**: 指标收集和监控
- **Grafana**: 数据可视化和仪表板
- **Node Exporter**: 节点监控
- **备份任务**: 自动化数据备份

## 部署顺序

### 1. 基础配置
```bash
# 创建命名空间
kubectl apply -f k8s/namespace.yaml

# 部署存储配置
kubectl apply -f k8s/storage.yaml

# 创建配置映射和密钥
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secrets.yaml
```

### 2. 基础设施服务
```bash
# 部署数据库
kubectl apply -f k8s/deployments/postgres.yaml

# 部署缓存
kubectl apply -f k8s/deployments/redis.yaml

# 部署消息队列
kubectl apply -f k8s/deployments/kafka.yaml

# 部署对象存储
kubectl apply -f k8s/deployments/minio.yaml

# 等待基础设施服务启动完成
kubectl wait --for=condition=ready pod -l app=postgres -n fruit-sort --timeout=300s
kubectl wait --for=condition=ready pod -l app=redis -n fruit-sort --timeout=300s
kubectl wait --for=condition=ready pod -l app=kafka -n fruit-sort --timeout=300s
kubectl wait --for=condition=ready pod -l app=minio -n fruit-sort --timeout=300s
```

### 3. 网关和应用服务
```bash
# 部署API网关
kubectl apply -f k8s/deployments/kong.yaml

# 等待Kong就绪
kubectl wait --for=condition=ready pod -l app=kong -n fruit-sort --timeout=300s

# 部署应用服务
kubectl apply -f k8s/deployments/ai-inference.yaml
kubectl apply -f k8s/deployments/upload-svc.yaml
kubectl apply -f k8s/deployments/query-svc.yaml
kubectl apply -f k8s/deployments/sort-svc.yaml
kubectl apply -f k8s/deployments/frontend.yaml
```

### 4. 网络和入口
```bash
# 部署Ingress配置
kubectl apply -f k8s/ingress.yaml
```

### 5. 监控系统
```bash
# 部署监控组件
kubectl apply -f k8s/monitoring.yaml
```

## 验证部署

### 检查服务状态
```bash
# 查看所有Pod状态
kubectl get pods -n fruit-sort

# 查看服务状态
kubectl get services -n fruit-sort

# 查看Ingress状态
kubectl get ingress -n fruit-sort

# 查看PVC状态
kubectl get pvc -n fruit-sort
```

### 健康检查
```bash
# 检查数据库连接
kubectl exec -it postgres-0 -n fruit-sort -- pg_isready -U postgres

# 检查Redis连接
kubectl exec -it redis-0 -n fruit-sort -- redis-cli ping

# 检查Kafka主题
kubectl exec -it kafka-0 -n fruit-sort -- kafka-topics --bootstrap-server localhost:9092 --list

# 检查MinIO状态
kubectl exec -it minio-0 -n fruit-sort -- mc admin info local
```

## 访问地址

### 生产环境访问
- **主应用**: https://fruit-sort.example.com
- **API网关**: https://api.fruit-sort.example.com
- **MinIO控制台**: https://minio.fruit-sort.example.com
- **Grafana监控**: https://grafana.fruit-sort.example.com

### 管理界面
- **Kong管理界面**: https://kong-admin.fruit-sort.example.com
- **Prometheus**: https://admin.fruit-sort.example.com/prometheus

## 扩容和维护

### 水平扩容
```bash
# 扩展应用服务副本
kubectl scale deployment ai-inference --replicas=3 -n fruit-sort
kubectl scale deployment query-service --replicas=3 -n fruit-sort
kubectl scale deployment sort-service --replicas=3 -n fruit-sort
```

### 更新应用
```bash
# 更新镜像版本
kubectl set image deployment/ai-inference ai-inference=your-registry/fruit-sort/ai-inference:v2.0.0 -n fruit-sort
kubectl rollout status deployment/ai-inference -n fruit-sort
```

### 备份和恢复
```bash
# 手动触发数据库备份
kubectl create job --from=cronjob/database-backup-job manual-backup-$(date +%s) -n fruit-sort

# 查看备份状态
kubectl logs job/manual-backup-xxxxx -n fruit-sort

# 恢复数据库（紧急情况）
kubectl exec -it database-backup-pod -n fruit-sort -- /scripts/restore-database.sh /backup/YYYY-MM-DD/database-HHMMSS.dump
```

## 安全配置

### 网络策略
- 已配置网络策略限制Pod间通信
- Ingress仅允许必要的外部访问
- 管理界面受IP白名单保护

### 认证和授权
- 管理界面使用基本认证
- API接口使用JWT令牌
- 服务间通信使用内部密钥

### 数据加密
- 传输中数据使用TLS加密
- 静态数据存储加密
- 密钥使用Kubernetes Secrets管理

## 监控和告警

### 关键指标
- 服务可用性和响应时间
- 资源使用率（CPU、内存、存储）
- 错误率和请求量
- 数据库和队列性能

### 告警规则
- 服务下线告警
- 高错误率告警
- 资源使用率过高告警
- 存储空间不足告警

## 故障排除

### 常见问题
1. **Pod启动失败**: 检查资源限制和镜像可用性
2. **服务无法访问**: 验证Service和NetworkPolicy配置
3. **数据库连接失败**: 检查密钥和网络连通性
4. **存储问题**: 验证StorageClass和PVC配置

### 日志查看
```bash
# 查看应用日志
kubectl logs -f deployment/ai-inference -n fruit-sort

# 查看系统事件
kubectl get events --sort-by=.metadata.creationTimestamp -n fruit-sort

# 查看Pod详细信息
kubectl describe pod <pod-name> -n fruit-sort
```

## 性能优化

### 资源调整
- 根据实际负载调整CPU和内存限制
- 启用HPA自动扩缩容
- 优化存储I/O性能

### 缓存优化
- 合理配置Redis缓存策略
- 使用CDN加速静态资源
- 启用应用级缓存

## 生产环境注意事项

1. **域名和证书**: 更新实际域名和SSL证书
2. **镜像仓库**: 配置私有镜像仓库认证
3. **资源配额**: 根据集群规模调整资源限制
4. **备份策略**: 配置定期备份和异地备份
5. **监控告警**: 配置邮件/短信告警通知
6. **安全扫描**: 定期扫描镜像和配置漏洞

## 支持和维护

- 定期更新组件版本
- 监控安全补丁发布
- 维护备份和恢复流程
- 定期测试灾难恢复计划
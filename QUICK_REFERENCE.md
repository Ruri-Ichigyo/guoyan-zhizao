# 快速参考 - 数据库集成功能

## 🎯 核心功能

### 1. 图像上传 → 数据库保存
```
用户上传图片 → 保存到uploads表 → AI推理 → 保存到results表 → 缓存到Redis
```

## 📊 数据表

### uploads表
存储上传的图片和状态
- id, filename, data (BYTEA)
- status: processing/completed/failed
- created_at, updated_at

### results表
存储AI检测结果
- upload_id (FK)
- detections (JSONB) - 检测详情
- total_fruits, grade_a, grade_b, grade_c
- avg_confidence
- created_at

## 🔌 API端点

### 后端API
- `POST /api/v1/uploads` - 上传图片
- `GET /api/v1/results` - 查询结果列表
- `GET /api/v1/results/:id` - 查询单个结果
- `GET /api/v1/statistics` - 获取统计数据（增强版）

### 统计API返回
```json
{
  "totals": {
    "total": 100,
    "completed": 95,
    "failed": 5,
    "total_fruits": 1234
  },
  "averages": {
    "avg_confidence": 0.95
  },
  "quality_distribution": {
    "grade_a": 800,
    "grade_b": 300,
    "grade_c": 134
  },
  "category_distribution": [
    { "name": "苹果", "value": 450 },
    { "name": "橙子", "value": 320 }
  ]
}
```

## 💻 前端集成

### 使用统计API
```typescript
import { statisticsApi } from '@/api/statistics'

// 获取统计数据
const stats = await statisticsApi.getStatistics()

// 更新页面
totalProcessed.value = stats.totals.total_fruits
accuracyRate.value = stats.averages.avg_confidence * 100
```

### 使用结果API
```typescript
import { resultApi } from '@/api/result'

// 获取结果列表
const results = await resultApi.getResults({
  page: 1,
  limit: 20,
  status: 'completed'
})
```

## 📝 待办事项

### 已完成 ✅
- [x] 后端数据保存逻辑
- [x] 统计API增强
- [x] 前端API封装
- [x] 结果查看页面集成
- [x] 文档编写

### 需要手动完成 ⚠️
- [ ] 更新DataDashboard.vue（参考：DATA_DASHBOARD_UPDATE_GUIDE.md）

## 🔍 快速测试

### 测试数据流
1. 上传一张图片
2. 检查数据库：`SELECT * FROM uploads;`
3. 检查结果：`SELECT * FROM results;`
4. 调用统计API：`curl http://localhost:3000/api/v1/statistics`
5. 查看数据大屏是否更新

### SQL快速查询
```sql
-- 查看上传状态分布
SELECT status, COUNT(*) FROM uploads GROUP BY status;

-- 查看检测结果统计
SELECT
    SUM(total_fruits) as 总果蔬数,
    SUM(grade_a) as A级,
    SUM(grade_b) as B级,
    SUM(grade_c) as C级,
    AVG(avg_confidence) as 平均置信度
FROM results;

-- 查看分类分布
SELECT
    detection->>'class_name' as 类别,
    COUNT(*) as 数量
FROM results,
     jsonb_array_elements(detections) as detection
GROUP BY detection->>'class_name'
ORDER BY 数量 DESC;
```

## 📚 相关文档
- 详细实现：`DATABASE_INTEGRATION_REPORT.md`
- 数据大屏更新：`DATA_DASHBOARD_UPDATE_GUIDE.md`
- 实时检测：`LIVE_DETECTION_FEATURE_REPORT.md`
- 首页更新：`HOMEPAGE_UPDATE_REPORT.md`

## 🚀 性能提示
- Redis缓存结果，减少数据库查询
- 使用JSONB索引加速查询
- 分页查询避免大量数据加载
- 定期清理旧数据

## 🔧 故障排查
1. **数据未保存**：检查数据库连接、事务是否提交
2. **统计数据不准确**：清除Redis缓存重新查询
3. **查询慢**：检查索引、考虑增加缓存
4. **AI推理失败**：检查AI服务状态、日志

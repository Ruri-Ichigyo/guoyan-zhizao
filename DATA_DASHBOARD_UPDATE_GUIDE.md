# 数据大屏组件更新说明

## 更新内容

### 1. 导入统计API
在脚本的导入部分添加：
```typescript
import { statisticsApi, type StatisticsData } from '@/api/statistics'
```

### 2. 添加加载真实数据的方法
在组件的methods部分添加：

```typescript
const loadRealStatistics = async () => {
  try {
    const data = await statisticsApi.getStatistics()

    // 更新统计数据
    totalProcessed.value = data.totals.total_fruits || 0
    accuracyRate.value = (data.averages.avg_confidence || 0) * 100
    defectiveCount.value = data.totals.failed || 0

    // 更新分类分布图表
    if (data.category_distribution && data.category_distribution.length > 0) {
      categoryChartOption.value.series[0].data = data.category_distribution
    }

    // 更新质量分析图表
    const qualityData = [
      data.quality_distribution.grade_a || 0,
      data.quality_distribution.grade_b || 0,
      data.quality_distribution.grade_c || 0,
      0 // 不合格 - 从failed中计算
    ]
    qualityChartOption.value.series[0].data = qualityData

    lastUpdateTime.value = new Date().toLocaleString()
  } catch (error) {
    console.error('Failed to load statistics:', error)
    ElMessage.error('加载统计数据失败')
  }
}
```

### 3. 在onMounted中调用
将onMounted函数更新为：

```typescript
onMounted(async () => {
  // 加载真实统计数据
  await loadRealStatistics()

  // Initialize charts with default data
  await Promise.all([
    updateVolumeChart(volumeTimeRange.value),
    updateAccuracyChart(accuracyTimeRange.value),
    updateCategoryChart(categoryTimeRange.value)
  ])

  // Start real-time updates
  statsUpdateInterval = window.setInterval(() => {
    updateStats()
    loadRealStatistics() // 每5秒刷新一次真实数据
  }, 5000)
})
```

### 4. 更新refreshQualityData方法
```typescript
const refreshQualityData = async () => {
  qualityChartLoading.value = true
  try {
    await loadRealStatistics()
    ElMessage.success('质量数据已刷新')
  } catch (error) {
    ElMessage.error('刷新质量数据失败')
  } finally {
    qualityChartLoading.value = false
  }
}
```

## 实现效果

- ✅ 数据大屏显示数据库中的真实数据
- ✅ 每5秒自动刷新统计信息
- ✅ 显示实时的总处理数、准确率等
- ✅ 分类分布图表显示实际检测到的果蔬类别
- ✅ 质量分析显示真实的A/B/C等级分布

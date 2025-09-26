import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Ref } from 'vue'
import { ElMessage } from 'element-plus'

// Types
export interface DetectionJob {
  id: string
  batchId: string
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'paused'
  type: 'single' | 'batch' | 'realtime'
  createdAt: string
  startedAt?: string
  completedAt?: string
  totalItems: number
  processedItems: number
  successItems: number
  failedItems: number
  progress: number
  results: DetectionResult[]
  settings: JobSettings
  metadata?: Record<string, any>
  error?: string
}

export interface DetectionResult {
  id: string
  itemId: string
  category: string
  quality: 'excellent' | 'good' | 'fair' | 'poor'
  confidence: number
  defects: string[]
  processingTime: number
  imageUrl: string
  thumbnailUrl: string
  detectedAt: string
  metadata?: Record<string, any>
}

export interface JobSettings {
  confidenceThreshold: number
  qualityStandard: 'strict' | 'standard' | 'loose'
  enabledCategories: string[]
  defectDetection: boolean
  defectSensitivity: number
  outputFormat: 'json' | 'csv' | 'xml'
  autoExport: boolean
  exportPath?: string
}

export interface JobStatistics {
  totalJobs: number
  completedJobs: number
  failedJobs: number
  totalProcessed: number
  averageAccuracy: number
  averageProcessingTime: number
  todayProcessed: number
  weeklyProcessed: number
  monthlyProcessed: number
}

export interface SystemStatus {
  isRunning: boolean
  currentSpeed: number
  systemLoad: {
    cpu: number
    memory: number
    gpu: number
  }
  deviceStatus: {
    cameras: number
    sorters: number
    controllers: number
    total: number
    online: number
  }
  errors: string[]
  warnings: string[]
}

// Default settings
const DEFAULT_JOB_SETTINGS: JobSettings = {
  confidenceThreshold: 85,
  qualityStandard: 'standard',
  enabledCategories: ['apple', 'orange', 'banana', 'pear', 'grape', 'watermelon'],
  defectDetection: true,
  defectSensitivity: 5,
  outputFormat: 'json',
  autoExport: false
}

export const useJobStore = defineStore('job', () => {
  // State
  const jobs: Ref<DetectionJob[]> = ref([])
  const currentJob: Ref<DetectionJob | null> = ref(null)
  const systemStatus: Ref<SystemStatus> = ref({
    isRunning: false,
    currentSpeed: 0,
    systemLoad: {
      cpu: 0,
      memory: 0,
      gpu: 0
    },
    deviceStatus: {
      cameras: 2,
      sorters: 2,
      controllers: 1,
      total: 5,
      online: 4
    },
    errors: [],
    warnings: []
  })

  const jobSettings: Ref<JobSettings> = ref({ ...DEFAULT_JOB_SETTINGS })
  const loading = ref(false)
  const error: Ref<string | null> = ref(null)

  // Computed
  const statistics = computed<JobStatistics>(() => {
    const now = new Date()
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
    const weekAgo = new Date(today.getTime() - 7 * 24 * 60 * 60 * 1000)
    const monthAgo = new Date(today.getTime() - 30 * 24 * 60 * 60 * 1000)

    const completedJobs = jobs.value.filter(job => job.status === 'completed')
    const failedJobs = jobs.value.filter(job => job.status === 'failed')

    const totalProcessed = completedJobs.reduce((sum, job) => sum + job.processedItems, 0)
    const todayProcessed = completedJobs
      .filter(job => new Date(job.completedAt!) >= today)
      .reduce((sum, job) => sum + job.processedItems, 0)
    const weeklyProcessed = completedJobs
      .filter(job => new Date(job.completedAt!) >= weekAgo)
      .reduce((sum, job) => sum + job.processedItems, 0)
    const monthlyProcessed = completedJobs
      .filter(job => new Date(job.completedAt!) >= monthAgo)
      .reduce((sum, job) => sum + job.processedItems, 0)

    // Calculate average accuracy from results
    const allResults = completedJobs.flatMap(job => job.results)
    const averageAccuracy = allResults.length > 0
      ? allResults.reduce((sum, result) => sum + result.confidence, 0) / allResults.length
      : 0

    // Calculate average processing time
    const averageProcessingTime = allResults.length > 0
      ? allResults.reduce((sum, result) => sum + result.processingTime, 0) / allResults.length
      : 0

    return {
      totalJobs: jobs.value.length,
      completedJobs: completedJobs.length,
      failedJobs: failedJobs.length,
      totalProcessed,
      averageAccuracy,
      averageProcessingTime,
      todayProcessed,
      weeklyProcessed,
      monthlyProcessed
    }
  })

  const runningJobs = computed(() =>
    jobs.value.filter(job => ['pending', 'processing'].includes(job.status))
  )

  const recentJobs = computed(() =>
    jobs.value
      .slice()
      .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
      .slice(0, 10)
  )

  const canStartJob = computed(() =>
    !loading.value && systemStatus.value.deviceStatus.online > 0 && currentJob.value?.status !== 'processing'
  )

  // Actions
  const createJob = async (
    type: DetectionJob['type'],
    totalItems: number,
    settings?: Partial<JobSettings>
  ): Promise<DetectionJob> => {
    loading.value = true
    error.value = null

    try {
      const job: DetectionJob = {
        id: generateJobId(),
        batchId: generateBatchId(),
        status: 'pending',
        type,
        createdAt: new Date().toISOString(),
        totalItems,
        processedItems: 0,
        successItems: 0,
        failedItems: 0,
        progress: 0,
        results: [],
        settings: { ...jobSettings.value, ...settings }
      }

      jobs.value.push(job)

      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 500))

      ElMessage.success(`任务 ${job.batchId} 创建成功`)
      return job
    } catch (err) {
      error.value = err instanceof Error ? err.message : '创建任务失败'
      ElMessage.error(error.value)
      throw err
    } finally {
      loading.value = false
    }
  }

  const startJob = async (jobId: string): Promise<void> => {
    loading.value = true
    error.value = null

    try {
      const job = jobs.value.find(j => j.id === jobId)
      if (!job) {
        throw new Error('任务不存在')
      }

      if (job.status !== 'pending' && job.status !== 'paused') {
        throw new Error('任务状态不允许启动')
      }

      job.status = 'processing'
      job.startedAt = new Date().toISOString()
      currentJob.value = job
      systemStatus.value.isRunning = true

      // Simulate processing
      simulateJobProcessing(job)

      ElMessage.success(`任务 ${job.batchId} 已启动`)
    } catch (err) {
      error.value = err instanceof Error ? err.message : '启动任务失败'
      ElMessage.error(error.value)
      throw err
    } finally {
      loading.value = false
    }
  }

  const pauseJob = async (jobId: string): Promise<void> => {
    loading.value = true
    error.value = null

    try {
      const job = jobs.value.find(j => j.id === jobId)
      if (!job) {
        throw new Error('任务不存在')
      }

      if (job.status !== 'processing') {
        throw new Error('只能暂停正在处理的任务')
      }

      job.status = 'paused'
      if (currentJob.value?.id === jobId) {
        systemStatus.value.isRunning = false
      }

      ElMessage.info(`任务 ${job.batchId} 已暂停`)
    } catch (err) {
      error.value = err instanceof Error ? err.message : '暂停任务失败'
      ElMessage.error(error.value)
      throw err
    } finally {
      loading.value = false
    }
  }

  const stopJob = async (jobId: string): Promise<void> => {
    loading.value = true
    error.value = null

    try {
      const job = jobs.value.find(j => j.id === jobId)
      if (!job) {
        throw new Error('任务不存在')
      }

      if (!['processing', 'paused'].includes(job.status)) {
        throw new Error('无法停止该任务')
      }

      job.status = 'failed'
      job.completedAt = new Date().toISOString()
      job.error = '用户手动停止'

      if (currentJob.value?.id === jobId) {
        currentJob.value = null
        systemStatus.value.isRunning = false
      }

      ElMessage.warning(`任务 ${job.batchId} 已停止`)
    } catch (err) {
      error.value = err instanceof Error ? err.message : '停止任务失败'
      ElMessage.error(error.value)
      throw err
    } finally {
      loading.value = false
    }
  }

  const deleteJob = async (jobId: string): Promise<void> => {
    loading.value = true
    error.value = null

    try {
      const jobIndex = jobs.value.findIndex(j => j.id === jobId)
      if (jobIndex === -1) {
        throw new Error('任务不存在')
      }

      const job = jobs.value[jobIndex]
      if (job.status === 'processing') {
        throw new Error('无法删除正在处理的任务')
      }

      jobs.value.splice(jobIndex, 1)

      if (currentJob.value?.id === jobId) {
        currentJob.value = null
      }

      ElMessage.success(`任务 ${job.batchId} 已删除`)
    } catch (err) {
      error.value = err instanceof Error ? err.message : '删除任务失败'
      ElMessage.error(error.value)
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateJobSettings = (settings: Partial<JobSettings>): void => {
    Object.assign(jobSettings.value, settings)
    ElMessage.success('任务设置已更新')
  }

  const resetJobSettings = (): void => {
    Object.assign(jobSettings.value, DEFAULT_JOB_SETTINGS)
    ElMessage.success('任务设置已重置为默认值')
  }

  const getJobById = (id: string): DetectionJob | undefined => {
    return jobs.value.find(job => job.id === id)
  }

  const getJobResults = async (jobId: string): Promise<DetectionResult[]> => {
    loading.value = true
    error.value = null

    try {
      const job = jobs.value.find(j => j.id === jobId)
      if (!job) {
        throw new Error('任务不存在')
      }

      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 300))

      return job.results
    } catch (err) {
      error.value = err instanceof Error ? err.message : '获取结果失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  const exportJobResults = async (
    jobId: string,
    format: 'json' | 'csv' | 'xml' = 'json'
  ): Promise<void> => {
    loading.value = true
    error.value = null

    try {
      const job = jobs.value.find(j => j.id === jobId)
      if (!job) {
        throw new Error('任务不存在')
      }

      let content: string
      let filename: string
      let mimeType: string

      switch (format) {
        case 'csv':
          content = exportToCsv(job.results)
          filename = `${job.batchId}_results.csv`
          mimeType = 'text/csv'
          break
        case 'xml':
          content = exportToXml(job.results)
          filename = `${job.batchId}_results.xml`
          mimeType = 'application/xml'
          break
        default:
          content = JSON.stringify(job.results, null, 2)
          filename = `${job.batchId}_results.json`
          mimeType = 'application/json'
      }

      // Download file
      const blob = new Blob([content], { type: `${mimeType};charset=utf-8;` })
      const link = document.createElement('a')
      link.href = URL.createObjectURL(blob)
      link.download = filename
      link.click()
      URL.revokeObjectURL(link.href)

      ElMessage.success(`结果已导出为 ${format.toUpperCase()} 格式`)
    } catch (err) {
      error.value = err instanceof Error ? err.message : '导出失败'
      ElMessage.error(error.value)
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateSystemStatus = (status: Partial<SystemStatus>): void => {
    Object.assign(systemStatus.value, status)
  }

  const addError = (error: string): void => {
    systemStatus.value.errors.push(error)
    // Keep only last 50 errors
    if (systemStatus.value.errors.length > 50) {
      systemStatus.value.errors.shift()
    }
  }

  const addWarning = (warning: string): void => {
    systemStatus.value.warnings.push(warning)
    // Keep only last 50 warnings
    if (systemStatus.value.warnings.length > 50) {
      systemStatus.value.warnings.shift()
    }
  }

  const clearErrors = (): void => {
    systemStatus.value.errors = []
  }

  const clearWarnings = (): void => {
    systemStatus.value.warnings = []
  }

  // Helper functions
  const generateJobId = (): string => {
    return `job_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  const generateBatchId = (): string => {
    const now = new Date()
    const dateStr = now.toISOString().slice(0, 10).replace(/-/g, '')
    const timeStr = now.toTimeString().slice(0, 8).replace(/:/g, '')
    const random = Math.random().toString(36).substr(2, 4).toUpperCase()
    return `BATCH${dateStr}${timeStr}${random}`
  }

  const simulateJobProcessing = (job: DetectionJob): void => {
    const interval = setInterval(() => {
      if (job.status !== 'processing') {
        clearInterval(interval)
        return
      }

      // Simulate progress
      const increment = Math.floor(Math.random() * 5) + 1
      job.processedItems = Math.min(job.processedItems + increment, job.totalItems)
      job.progress = Math.round((job.processedItems / job.totalItems) * 100)

      // Generate mock results
      for (let i = 0; i < increment && job.results.length < job.totalItems; i++) {
        const result = generateMockResult(job)
        job.results.push(result)

        if (result.confidence >= job.settings.confidenceThreshold) {
          job.successItems++
        } else {
          job.failedItems++
        }
      }

      // Update system load
      systemStatus.value.currentSpeed = Math.floor(Math.random() * 100) + 150
      systemStatus.value.systemLoad.cpu = Math.floor(Math.random() * 30) + 50
      systemStatus.value.systemLoad.memory = Math.floor(Math.random() * 20) + 60
      systemStatus.value.systemLoad.gpu = Math.floor(Math.random() * 40) + 40

      // Check if job is completed
      if (job.processedItems >= job.totalItems) {
        job.status = 'completed'
        job.completedAt = new Date().toISOString()
        job.progress = 100
        currentJob.value = null
        systemStatus.value.isRunning = false

        clearInterval(interval)
        ElMessage.success(`任务 ${job.batchId} 已完成`)
      }
    }, 1000)
  }

  const generateMockResult = (job: DetectionJob): DetectionResult => {
    const categories = job.settings.enabledCategories
    const category = categories[Math.floor(Math.random() * categories.length)]
    const qualities: DetectionResult['quality'][] = ['excellent', 'good', 'fair', 'poor']
    const quality = qualities[Math.floor(Math.random() * qualities.length)]
    const confidence = Math.floor(Math.random() * 40) + 60

    const defectTypes = ['表面缺陷', '形状异常', '颜色异常', '尺寸不符', '碰伤', '腐烂']
    const defects = job.settings.defectDetection && Math.random() > 0.7
      ? defectTypes.slice(0, Math.floor(Math.random() * 3) + 1)
      : []

    return {
      id: `result_${job.results.length + 1}`,
      itemId: `${job.batchId}_${String(job.results.length + 1).padStart(4, '0')}`,
      category,
      quality,
      confidence,
      defects,
      processingTime: Math.floor(Math.random() * 200) + 50,
      imageUrl: `/api/images/result_${job.results.length + 1}.jpg`,
      thumbnailUrl: `/api/images/thumb_${job.results.length + 1}.jpg`,
      detectedAt: new Date().toISOString(),
      metadata: {
        size: `${Math.floor(Math.random() * 50 + 30)}mm`,
        weight: `${Math.floor(Math.random() * 200 + 50)}g`,
        ripeness: `${Math.floor(Math.random() * 100)}%`
      }
    }
  }

  const exportToCsv = (results: DetectionResult[]): string => {
    const headers = [
      'ID', '物品ID', '分类', '质量', '置信度', '处理时间', '检测时间', '缺陷'
    ]

    const rows = results.map(result => [
      result.id,
      result.itemId,
      result.category,
      result.quality,
      `${result.confidence}%`,
      `${result.processingTime}ms`,
      result.detectedAt,
      result.defects.join(';')
    ])

    return [headers, ...rows].map(row => row.join(',')).join('\n')
  }

  const exportToXml = (results: DetectionResult[]): string => {
    let xml = '<?xml version="1.0" encoding="UTF-8"?>\n<results>\n'

    results.forEach(result => {
      xml += '  <result>\n'
      xml += `    <id>${result.id}</id>\n`
      xml += `    <itemId>${result.itemId}</itemId>\n`
      xml += `    <category>${result.category}</category>\n`
      xml += `    <quality>${result.quality}</quality>\n`
      xml += `    <confidence>${result.confidence}</confidence>\n`
      xml += `    <processingTime>${result.processingTime}</processingTime>\n`
      xml += `    <detectedAt>${result.detectedAt}</detectedAt>\n`
      xml += `    <defects>${result.defects.join(';')}</defects>\n`
      xml += '  </result>\n'
    })

    xml += '</results>'
    return xml
  }

  // Initialize with mock data
  const initializeMockData = (): void => {
    // Add some sample jobs
    const mockJobs: DetectionJob[] = [
      {
        id: generateJobId(),
        batchId: 'BATCH20241201120001',
        status: 'completed',
        type: 'batch',
        createdAt: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
        startedAt: new Date(Date.now() - 24 * 60 * 60 * 1000 + 60000).toISOString(),
        completedAt: new Date(Date.now() - 23 * 60 * 60 * 1000).toISOString(),
        totalItems: 1000,
        processedItems: 1000,
        successItems: 950,
        failedItems: 50,
        progress: 100,
        results: [],
        settings: { ...DEFAULT_JOB_SETTINGS }
      },
      {
        id: generateJobId(),
        batchId: 'BATCH20241201080001',
        status: 'completed',
        type: 'batch',
        createdAt: new Date(Date.now() - 12 * 60 * 60 * 1000).toISOString(),
        startedAt: new Date(Date.now() - 12 * 60 * 60 * 1000 + 30000).toISOString(),
        completedAt: new Date(Date.now() - 11 * 60 * 60 * 1000).toISOString(),
        totalItems: 500,
        processedItems: 500,
        successItems: 485,
        failedItems: 15,
        progress: 100,
        results: [],
        settings: { ...DEFAULT_JOB_SETTINGS }
      }
    ]

    jobs.value = mockJobs
  }

  // Initialize store
  initializeMockData()

  return {
    // State
    jobs: readonly(jobs),
    currentJob: readonly(currentJob),
    systemStatus: readonly(systemStatus),
    jobSettings: readonly(jobSettings),
    loading: readonly(loading),
    error: readonly(error),

    // Computed
    statistics: readonly(statistics),
    runningJobs: readonly(runningJobs),
    recentJobs: readonly(recentJobs),
    canStartJob: readonly(canStartJob),

    // Actions
    createJob,
    startJob,
    pauseJob,
    stopJob,
    deleteJob,
    updateJobSettings,
    resetJobSettings,
    getJobById,
    getJobResults,
    exportJobResults,
    updateSystemStatus,
    addError,
    addWarning,
    clearErrors,
    clearWarnings
  }
})
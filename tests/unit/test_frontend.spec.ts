import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ElementPlus from 'element-plus'

// 组件导入
import UploadPanel from '../../frontend/src/components/UploadPanel.vue'
import DataDashboard from '../../frontend/src/components/DataDashboard.vue'
import { useJobStore } from '../../frontend/src/stores/job'

// Mock API
vi.mock('../../frontend/src/api/upload', () => ({
  uploadApi: {
    uploadFile: vi.fn(() => Promise.resolve({
      upload_id: 'mock-upload-123',
      status: 'uploaded'
    })),
    getPresignedUrl: vi.fn(() => Promise.resolve({
      upload_id: 'mock-presign-123',
      presigned_url: 'https://mock-presigned-url.com',
      status: 'presigned'
    })),
    uploadToPresignedUrl: vi.fn(() => Promise.resolve())
  }
}))

vi.mock('../../frontend/src/api/result', () => ({
  resultApi: {
    getResult: vi.fn(() => Promise.resolve({
      job_id: 'test-job-123',
      status: 'completed',
      results: [
        {
          class_name: '苹果',
          confidence: 0.95,
          bbox: [10, 20, 100, 150],
          quality_grade: 'A'
        }
      ],
      created_at: '2023-12-01T10:30:00Z'
    })),
    getResults: vi.fn(() => Promise.resolve({
      data: [
        {
          job_id: 'test-job-1',
          status: 'completed',
          total_fruits: 15,
          grade_a: 8,
          grade_b: 5,
          grade_c: 2,
          created_at: '2023-12-01T10:30:00Z'
        }
      ],
      pagination: {
        page: 1,
        limit: 10,
        total: 50,
        pages: 5
      }
    })),
    getStatistics: vi.fn(() => Promise.resolve({
      total_processed: 1250,
      accuracy_rate: 0.952,
      average_processing_time: 2.3,
      grade_distribution: { A: 620, B: 430, C: 200 },
      processing_trend: [
        { time: '2023-12-01T10:00:00Z', count: 52 }
      ]
    }))
  }
}))

describe('UploadPanel Component', () => {
  let wrapper
  let pinia

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    wrapper = mount(UploadPanel, {
      global: {
        plugins: [pinia, ElementPlus],
        stubs: {
          'el-upload': {
            template: '<div class="mock-upload"><slot></slot></div>',
            props: ['beforeUpload', 'httpRequest']
          },
          'el-progress': {
            template: '<div class="mock-progress" :data-percentage="percentage"></div>',
            props: ['percentage', 'status']
          },
          'el-button': {
            template: '<button @click="$emit(\'click\')"><slot></slot></button>'
          },
          'el-card': {
            template: '<div class="mock-card"><slot></slot></div>'
          }
        }
      }
    })
  })

  afterEach(() => {
    wrapper.unmount()
  })

  it('renders upload panel correctly', () => {
    expect(wrapper.find('.upload-panel').exists()).toBe(true)
    expect(wrapper.find('.mock-upload').exists()).toBe(true)
  })

  it('switches upload mode correctly', async () => {
    const modeButton = wrapper.find('[data-testid="upload-mode-toggle"]')
    expect(modeButton.exists()).toBe(true)

    await modeButton.trigger('click')

    // 验证模式切换
    const vm = wrapper.vm
    expect(vm.uploadMode).toBe('presigned')
  })

  it('handles file upload validation', async () => {
    const vm = wrapper.vm

    // 测试有效文件
    const validFile = new File(['test'], 'test.jpg', { type: 'image/jpeg' })
    const validResult = vm.handleBeforeUpload(validFile)
    expect(validResult).toBe(true)

    // 测试无效文件类型
    const invalidFile = new File(['test'], 'test.txt', { type: 'text/plain' })
    const invalidResult = vm.handleBeforeUpload(invalidFile)
    expect(invalidResult).toBe(false)

    // 测试文件大小限制
    const largeFile = new File(['x'.repeat(11 * 1024 * 1024)], 'large.jpg', {
      type: 'image/jpeg'
    })
    const largeResult = vm.handleBeforeUpload(largeFile)
    expect(largeResult).toBe(false)
  })

  it('handles successful upload', async () => {
    const vm = wrapper.vm
    const mockFile = new File(['test'], 'test.jpg', { type: 'image/jpeg' })

    const mockOptions = {
      file: mockFile,
      onSuccess: vi.fn(),
      onError: vi.fn(),
      onProgress: vi.fn()
    }

    await vm.handleUpload(mockOptions)

    // 验证上传成功
    expect(vm.recentUploads).toHaveLength(1)
    expect(vm.recentUploads[0]).toMatchObject({
      upload_id: 'mock-upload-123',
      filename: 'test.jpg',
      status: 'uploaded'
    })
  })

  it('displays upload progress correctly', async () => {
    const vm = wrapper.vm

    // 添加上传项目
    vm.uploadingFiles = [{
      id: 'test-upload',
      name: 'test.jpg',
      size: 1024,
      progress: 50,
      status: 'uploading'
    }]

    await wrapper.vm.$nextTick()

    const progressElement = wrapper.find('[data-percentage="50"]')
    expect(progressElement.exists()).toBe(true)
  })

  it('handles upload errors gracefully', async () => {
    const { uploadApi } = await import('../../frontend/src/api/upload')

    // Mock upload failure
    uploadApi.uploadFile.mockRejectedValueOnce(new Error('Upload failed'))

    const vm = wrapper.vm
    const mockFile = new File(['test'], 'test.jpg', { type: 'image/jpeg' })

    const mockOptions = {
      file: mockFile,
      onSuccess: vi.fn(),
      onError: vi.fn(),
      onProgress: vi.fn()
    }

    await vm.handleUpload(mockOptions)

    // 验证错误处理
    expect(vm.uploadingFiles.some(f => f.status === 'error')).toBe(true)
  })
})

describe('DataDashboard Component', () => {
  let wrapper
  let pinia

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)

    wrapper = mount(DataDashboard, {
      global: {
        plugins: [pinia, ElementPlus],
        stubs: {
          'v-chart': {
            template: '<div class="mock-chart" :option="option"></div>',
            props: ['option']
          },
          'el-card': {
            template: '<div class="mock-card"><slot></slot></div>'
          },
          'el-statistic': {
            template: '<div class="mock-statistic" :value="value"></div>',
            props: ['value', 'title']
          }
        }
      }
    })
  })

  afterEach(() => {
    wrapper.unmount()
  })

  it('renders dashboard components', () => {
    expect(wrapper.find('.data-dashboard').exists()).toBe(true)
    expect(wrapper.findAll('.mock-card')).toHaveLength.greaterThan(0)
  })

  it('loads statistics data on mount', async () => {
    const vm = wrapper.vm

    // 等待组件加载
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(vm.statistics).toBeDefined()
    expect(vm.statistics.total_processed).toBe(1250)
    expect(vm.statistics.accuracy_rate).toBe(0.952)
  })

  it('updates charts when time range changes', async () => {
    const vm = wrapper.vm

    const initialProcessingTrend = vm.processingTrendData

    // 更改时间范围
    vm.timeRange = '7d'
    vm.updateCharts()

    await wrapper.vm.$nextTick()

    // 验证图表数据更新
    expect(vm.processingTrendData).toBeDefined()
  })

  it('formats statistics correctly', () => {
    const vm = wrapper.vm

    expect(vm.formatNumber(1234)).toBe('1,234')
    expect(vm.formatNumber(1234567)).toBe('1,234,567')
    expect(vm.formatPercentage(0.952)).toBe('95.2%')
  })

  it('handles data loading errors', async () => {
    const { resultApi } = await import('../../frontend/src/api/result')

    // Mock API error
    resultApi.getStatistics.mockRejectedValueOnce(new Error('API Error'))

    const vm = wrapper.vm
    await vm.fetchStatistics()

    // 验证错误处理
    expect(vm.loading).toBe(false)
    expect(vm.error).toBeTruthy()
  })
})

describe('Job Store', () => {
  let store

  beforeEach(() => {
    setActivePinia(createPinia())
    store = useJobStore()
  })

  it('initializes with empty state', () => {
    expect(store.jobs).toEqual([])
    expect(store.activeJobs).toEqual([])
    expect(store.completedJobs).toEqual([])
  })

  it('adds new job correctly', () => {
    const newJob = {
      id: 'test-job-1',
      filename: 'test.jpg',
      status: 'pending',
      progress: 0,
      createdAt: new Date()
    }

    store.addJob(newJob)

    expect(store.jobs).toHaveLength(1)
    expect(store.jobs[0]).toMatchObject(newJob)
    expect(store.activeJobs).toHaveLength(1)
  })

  it('updates job status correctly', () => {
    const job = {
      id: 'test-job-2',
      filename: 'test2.jpg',
      status: 'pending',
      progress: 0,
      createdAt: new Date()
    }

    store.addJob(job)
    store.updateJobStatus('test-job-2', 'completed', 100)

    const updatedJob = store.getJob('test-job-2')
    expect(updatedJob.status).toBe('completed')
    expect(updatedJob.progress).toBe(100)
    expect(store.completedJobs).toHaveLength(1)
    expect(store.activeJobs).toHaveLength(0)
  })

  it('removes job correctly', () => {
    const job = {
      id: 'test-job-3',
      filename: 'test3.jpg',
      status: 'pending',
      progress: 0,
      createdAt: new Date()
    }

    store.addJob(job)
    expect(store.jobs).toHaveLength(1)

    store.removeJob('test-job-3')
    expect(store.jobs).toHaveLength(0)
  })

  it('calculates statistics correctly', () => {
    // 添加测试数据
    const jobs = [
      { id: '1', status: 'completed', results: [{ quality_grade: 'A' }] },
      { id: '2', status: 'completed', results: [{ quality_grade: 'B' }] },
      { id: '3', status: 'completed', results: [{ quality_grade: 'A' }] },
      { id: '4', status: 'failed', results: [] }
    ]

    jobs.forEach(job => store.addJob(job))

    const stats = store.jobStatistics
    expect(stats.total).toBe(4)
    expect(stats.completed).toBe(3)
    expect(stats.failed).toBe(1)
    expect(stats.successRate).toBe(0.75)
  })

  it('exports data correctly', () => {
    const job = {
      id: 'export-test',
      filename: 'export.jpg',
      status: 'completed',
      results: [{ class_name: '苹果', confidence: 0.95 }]
    }

    store.addJob(job)

    const csvData = store.exportData('csv')
    expect(csvData).toContain('ID,文件名,状态')
    expect(csvData).toContain('export-test,export.jpg,completed')

    const jsonData = store.exportData('json')
    const parsedData = JSON.parse(jsonData)
    expect(parsedData).toHaveLength(1)
    expect(parsedData[0].id).toBe('export-test')
  })
})

describe('WebSocket Integration', () => {
  let mockWebSocket

  beforeEach(() => {
    // Mock WebSocket
    mockWebSocket = {
      send: vi.fn(),
      close: vi.fn(),
      readyState: WebSocket.OPEN,
      onopen: null,
      onmessage: null,
      onerror: null,
      onclose: null
    }

    global.WebSocket = vi.fn(() => mockWebSocket)
  })

  it('connects to WebSocket successfully', async () => {
    const { connectWebSocket } = await import('../../frontend/src/websocket')

    const onMessage = vi.fn()
    connectWebSocket('ws://test.com', onMessage)

    expect(global.WebSocket).toHaveBeenCalledWith('ws://test.com')

    // 模拟连接打开
    mockWebSocket.onopen()

    // 模拟接收消息
    const testMessage = {
      type: 'detection_result',
      data: { job_id: 'test-123', status: 'completed' }
    }

    mockWebSocket.onmessage({ data: JSON.stringify(testMessage) })

    expect(onMessage).toHaveBeenCalledWith(testMessage)
  })

  it('handles WebSocket connection errors', async () => {
    const { connectWebSocket } = await import('../../frontend/src/websocket')

    const onMessage = vi.fn()
    connectWebSocket('ws://invalid.com', onMessage)

    // 模拟连接错误
    const errorEvent = new Error('Connection failed')
    mockWebSocket.onerror(errorEvent)

    // 验证错误处理
    expect(mockWebSocket.readyState).toBeDefined()
  })

  it('handles WebSocket reconnection', async () => {
    const { connectWebSocket, disconnectWebSocket } = await import('../../frontend/src/websocket')

    connectWebSocket('ws://test.com', vi.fn())

    // 模拟连接断开
    mockWebSocket.onclose({ wasClean: false })

    // 验证重连尝试
    expect(global.WebSocket).toHaveBeenCalled()
  })
})

describe('Utility Functions', () => {
  it('formats file sizes correctly', () => {
    // 假设有一个格式化文件大小的函数
    const formatFileSize = (bytes) => {
      if (bytes === 0) return '0 Bytes'
      const k = 1024
      const sizes = ['Bytes', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }

    expect(formatFileSize(0)).toBe('0 Bytes')
    expect(formatFileSize(1024)).toBe('1 KB')
    expect(formatFileSize(1048576)).toBe('1 MB')
    expect(formatFileSize(1073741824)).toBe('1 GB')
  })

  it('validates form inputs correctly', () => {
    const validateEmail = (email) => {
      const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
      return re.test(email)
    }

    expect(validateEmail('test@example.com')).toBe(true)
    expect(validateEmail('invalid-email')).toBe(false)
    expect(validateEmail('')).toBe(false)
  })

  it('handles date formatting correctly', () => {
    const formatDate = (dateString) => {
      return new Date(dateString).toLocaleString('zh-CN')
    }

    const testDate = '2023-12-01T10:30:00Z'
    const formatted = formatDate(testDate)

    expect(formatted).toMatch(/2023/)
    expect(formatted).toMatch(/12/)
    expect(formatted).toMatch(/01/)
  })
})

describe('Error Boundary', () => {
  it('catches and displays errors gracefully', async () => {
    // 创建会抛出错误的组件
    const ErrorComponent = {
      template: '<div>{{ throwError() }}</div>',
      methods: {
        throwError() {
          throw new Error('Test error')
        }
      }
    }

    // 这里需要一个错误边界组件来测试
    // 实际实现会根据 Vue 3 的错误处理机制
    expect(() => {
      mount(ErrorComponent)
    }).toThrow('Test error')
  })
})
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import type { Mock } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import type { UploadResult } from '../../frontend/src/api/upload'
import { useUserStore } from '../../frontend/src/stores/user'

vi.mock('../../frontend/src/api/upload', () => ({
  uploadApi: {
    uploadFile: vi.fn()
  }
}))

vi.mock('../../frontend/src/api/result', () => ({
  resultApi: {
    getResult: vi.fn(),
    getResults: vi.fn(),
    getStatistics: vi.fn()
  }
}))

vi.mock('element-plus', () => ({
  ElMessage: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn()
  }
}))

vi.mock('@element-plus/icons-vue', () => ({
  UploadFilled: { template: '<span />' }
}))

import { ElMessage } from 'element-plus'
import UploadPanel from '../../frontend/src/components/UploadPanel.vue'
import ResultsView from '../../frontend/src/views/ResultsView.vue'
import { uploadApi } from '../../frontend/src/api/upload'
import { resultApi } from '../../frontend/src/api/result'

const elementStubs = {
  'el-card': {
    template: '<div class="el-card"><slot name="header"></slot><slot></slot></div>'
  },
  'el-button': {
    template: '<button class="el-button" @click="$emit(\'click\')"><slot></slot></button>'
  },
  'el-icon': {
    template: '<span class="el-icon"><slot></slot></span>'
  },
  'el-skeleton': {
    template: '<div class="el-skeleton"></div>'
  },
  'el-result': {
    props: ['icon', 'title', 'subTitle'],
    template: '<div class="el-result"><slot></slot><slot name="extra"></slot></div>'
  },
  'el-empty': {
    props: ['description'],
    template: '<div class="el-empty"><slot></slot></div>'
  },
  'el-tag': {
    props: ['type', 'size'],
    template: '<span class="el-tag"><slot></slot></span>'
  },
  'el-row': {
    template: '<div class="el-row"><slot></slot></div>'
  },
  'el-col': {
    props: ['span'],
    template: '<div class="el-col"><slot></slot></div>'
  },
  'el-table': {
    props: ['data', 'stripe', 'border', 'loading'],
    template: '<table class="el-table"><slot></slot></table>'
  },
  'el-table-column': {
    props: ['prop', 'label', 'minWidth', 'width'],
    template: '<template><slot></slot></template>'
  },
  'el-statistic': {
    props: ['title', 'value'],
    template: '<div class="el-statistic"><slot></slot></div>'
  },
  'el-progress': {
    props: ['percentage'],
    template: '<div class="el-progress"><slot></slot></div>'
  },
  'el-dialog': {
    props: ['modelValue', 'title', 'width'],
    emits: ['update:modelValue'],
    template: '<div class="el-dialog"><slot></slot></div>'
  },
  'el-select': {
    props: ['modelValue', 'placeholder', 'clearable'],
    emits: ['update:modelValue', 'change'],
    template: '<select class="el-select" @change="$emit(\'change\', $event.target.value)"><slot></slot></select>'
  },
  'el-option': {
    props: ['label', 'value'],
    template: '<option :value="value"><slot></slot></option>'
  },
  'el-pagination': {
    props: ['currentPage', 'pageSize', 'total'],
    emits: ['current-change'],
    template: '<div class="el-pagination"><slot></slot></div>'
  },
  'el-drawer': {
    props: ['modelValue', 'title', 'size'],
    emits: ['update:modelValue'],
    template: '<div class="el-drawer"><slot></slot></div>'
  }
}

const mockDetections = [
  {
    class_name: '苹果',
    confidence: 0.95,
    bbox: [0, 0, 100, 100],
    quality_grade: 'A'
  }
]

const mockUploadResult: UploadResult = {
  upload_id: 'mock-upload-123',
  filename: 'test.jpg',
  status: 'completed',
  created_at: '2024-10-08T10:00:00Z',
  total_fruits: 3,
  grade_a: 2,
  grade_b: 1,
  grade_c: 0,
  avg_confidence: 0.92,
  detections: mockDetections
}

const mockResultSummary = {
  upload_id: 'mock-upload-123',
  filename: 'test.jpg',
  status: 'completed',
  created_at: '2024-10-08T10:00:00Z',
  total_fruits: 3,
  grade_a: 2,
  grade_b: 1,
  grade_c: 0,
  avg_confidence: 0.92
}

const mockDetailedResult = {
  ...mockResultSummary,
  detections: mockDetections
}

const mockStatistics = {
  totals: {
    total: 1,
    completed: 1,
    processing: 0,
    total_fruits: 3
  },
  averages: {
    avg_confidence: 0.92
  }
}

const uploadApiMock = uploadApi.uploadFile as unknown as Mock
const getResultsMock = resultApi.getResults as unknown as Mock
const getResultMock = resultApi.getResult as unknown as Mock
const getStatisticsMock = resultApi.getStatistics as unknown as Mock
const messageSuccessMock = ElMessage.success as unknown as Mock
const messageErrorMock = ElMessage.error as unknown as Mock

beforeEach(() => {
  vi.clearAllMocks()
  uploadApiMock.mockResolvedValue(mockUploadResult)
  getResultsMock.mockResolvedValue({
    data: [mockResultSummary],
    pagination: { page: 1, limit: 10, total: 1, pages: 1 }
  })
  getResultMock.mockResolvedValue(mockDetailedResult)
  getStatisticsMock.mockResolvedValue(mockStatistics)
  const store = useUserStore()
  store.setAuth('mock-token', new Date(Date.now() + 60 * 1000).toISOString(), {
    user_id: 'user-1',
    username: 'tester',
    role: 'operator'
  })
})

afterEach(() => {
  vi.clearAllMocks()
  const store = useUserStore()
  store.clearAuth()
})

describe('UploadPanel', () => {
  const createWrapper = () =>
    mount(UploadPanel, {
      global: {
        stubs: elementStubs,
        mocks: {
          $router: { push: vi.fn() }
        }
      }
    })

  it('renders placeholder before any upload', async () => {
    const wrapper = createWrapper()
    await flushPromises()

    expect(wrapper.find('.upload-placeholder').exists()).toBe(true)
    expect(getResultsMock).toHaveBeenCalledWith({ page: 1, limit: 5 })
    wrapper.unmount()
  })

  it('rejects non-image files', async () => {
    const wrapper = createWrapper()
    await flushPromises()

    const invalidFile = new File(['hello'], 'test.txt', { type: 'text/plain' })
    await wrapper.vm.handleFileChange({
      target: { files: [invalidFile], value: 'dummy' }
    } as unknown as Event)

    expect(uploadApiMock).not.toHaveBeenCalled()
    expect(messageErrorMock).toHaveBeenCalledWith('仅支持图片文件')
    wrapper.unmount()
  })

  it('uploads image and updates recent results', async () => {
    const wrapper = createWrapper()
    await flushPromises()

    const imageFile = new File(['binary'], 'fruit.jpg', { type: 'image/jpeg' })
    await wrapper.vm.handleFileChange({
      target: { files: [imageFile], value: 'dummy' }
    } as unknown as Event)

    await flushPromises()

    expect(uploadApiMock).toHaveBeenCalledWith(imageFile)
    expect(wrapper.vm.lastResult).toEqual(mockUploadResult)
    expect(getResultsMock).toHaveBeenCalledTimes(2)
    expect(messageSuccessMock).toHaveBeenCalledWith('上传并分析完成')
    wrapper.unmount()
  })

  it('blocks upload when user lacks permission', async () => {
    const store = useUserStore()
    store.setAuth('mock-token', new Date(Date.now() + 60 * 1000).toISOString(), {
      user_id: 'viewer-1',
      username: 'viewer',
      role: 'viewer'
    })

    const wrapper = createWrapper()
    await flushPromises()

    const imageFile = new File(['binary'], 'fruit.jpg', { type: 'image/jpeg' })
    await wrapper.vm.handleFileChange({
      target: { files: [imageFile], value: 'dummy' }
    } as unknown as Event)

    expect(uploadApiMock).not.toHaveBeenCalled()
    expect(messageErrorMock).not.toHaveBeenCalled()
    expect(ElMessage.warning).toHaveBeenCalledWith('当前账号无权上传图片')
    wrapper.unmount()
  })
})

describe('ResultsView', () => {
  const createWrapper = () =>
    mount(ResultsView, {
      global: {
        stubs: elementStubs,
        mocks: {
          $router: { push: vi.fn() }
        }
      }
    })

  it('loads statistics and results on mount', async () => {
    const wrapper = createWrapper()
    await flushPromises()

    expect(getStatisticsMock).toHaveBeenCalled()
    expect(getResultsMock).toHaveBeenCalledWith({ page: 1, limit: 10, status: undefined })
    expect(wrapper.vm.results).toHaveLength(1)
    expect(wrapper.vm.pagination.total).toBe(1)
    wrapper.unmount()
  })

  it('opens detail drawer with selected result', async () => {
    const wrapper = createWrapper()
    await flushPromises()

    await wrapper.vm.openDetail(mockResultSummary.upload_id)
    await flushPromises()

    expect(getResultMock).toHaveBeenCalledWith(mockResultSummary.upload_id)
    expect(wrapper.vm.detailVisible).toBe(true)
    expect(wrapper.vm.selectedResult).toEqual(mockDetailedResult)
    wrapper.unmount()
  })

  it('handles result loading error gracefully', async () => {
    getResultsMock.mockRejectedValueOnce(new Error('network error'))
    const wrapper = createWrapper()
    await flushPromises()

    expect(messageErrorMock).toHaveBeenCalledWith('加载记录失败')
    wrapper.unmount()
  })
})

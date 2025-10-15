import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface Job {
  id: string
  filename: string
  status: string
  progress: number
  createdAt: Date
  result?: any
}

export const useJobStore = defineStore('job', () => {
  const jobs = ref<Job[]>([])
  const currentJob = ref<Job | null>(null)

  const addJob = (job: Job) => {
    jobs.value.push(job)
  }

  const updateJob = (id: string, updates: Partial<Job>) => {
    const job = jobs.value.find(j => j.id === id)
    if (job) {
      Object.assign(job, updates)
    }
  }

  const removeJob = (id: string) => {
    const index = jobs.value.findIndex(j => j.id === id)
    if (index !== -1) {
      jobs.value.splice(index, 1)
    }
  }

  const getJob = (id: string) => {
    return jobs.value.find(j => j.id === id)
  }

  const activeJobs = computed(() => {
    return jobs.value.filter(j => j.status === 'processing' || j.status === 'pending')
  })

  return {
    jobs,
    currentJob,
    activeJobs,
    addJob,
    updateJob,
    removeJob,
    getJob
  }
})

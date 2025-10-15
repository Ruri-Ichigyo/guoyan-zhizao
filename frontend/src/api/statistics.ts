import { http } from './http'

export interface StatisticsData {
  totals: Record<string, number>
  averages: Record<string, number>
  quality_distribution: Record<string, number>
  category_distribution: Array<{
    name: string
    value: number
  }>
}

export const statisticsApi = {
  async getStatistics(): Promise<StatisticsData> {
    const response = await http.get('/statistics')
    return response.data
  }
}

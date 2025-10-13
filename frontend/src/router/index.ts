import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Home',
      component: () => import('../views/HomeView.vue')
    },
    {
      path: '/upload',
      name: 'Upload',
      component: () => import('../components/UploadPanel.vue')
    },
    {
      path: '/camera',
      name: 'Camera',
      component: () => import('../components/LiveCamera.vue')
    },
    {
      path: '/dashboard',
      name: 'Dashboard',
      component: () => import('../components/DataDashboard.vue')
    },
    {
      path: '/results',
      name: 'Results',
      component: () => import('../views/ResultsView.vue')
    },
    {
      path: '/sort',
      name: 'Sort',
      component: () => import('../views/SortView.vue')
    }
  ]
})

export default router

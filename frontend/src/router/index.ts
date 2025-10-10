import { createRouter, createWebHistory } from 'vue-router'

import { useUserStore } from '@/stores/user'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Home',
      component: () => import('../views/HomeView.vue'),
      meta: {
        requiresAuth: true,
        roles: ['admin', 'operator', 'viewer']
      }
    },
    {
      path: '/upload',
      name: 'Upload',
      component: () => import('../components/UploadPanel.vue'),
      meta: {
        requiresAuth: true,
        roles: ['admin', 'operator']
      }
    },
    {
      path: '/results',
      name: 'Results',
      component: () => import('../views/ResultsView.vue'),
      meta: {
        requiresAuth: true,
        roles: ['admin', 'operator', 'viewer']
      }
    },
    {
      path: '/login',
      name: 'Login',
      component: () => import('../views/LoginView.vue')
    },
    {
      path: '/forbidden',
      name: 'Forbidden',
      component: () => import('../views/ForbiddenView.vue'),
      meta: {
        requiresAuth: true
      }
    }
  ]
})

router.beforeEach((to) => {
  const store = useUserStore()

  if (to.name === 'Login') {
    return true
  }

  const requiresAuth = (to.meta.requiresAuth as boolean | undefined) ?? false
  const hasSession = store.ensureSession()

  if (requiresAuth && !hasSession) {
    store.clearAuth()
    return {
      name: 'Login',
      query: { redirect: to.fullPath }
    }
  }

  const allowedRoles = to.meta.roles as string[] | undefined
  if (allowedRoles && allowedRoles.length) {
    const role = store.role.value.toLowerCase()
    if (!role || !allowedRoles.map((value) => value.toLowerCase()).includes(role)) {
      return { name: 'Forbidden' }
    }
  }

  return true
})

export default router

<template>
  <div class="app-shell">
    <header class="app-header">
      <div>
        <h1>果蔬智造核心平台</h1>
        <p class="subtitle">上传识别 · 智能分析 · 快速查询</p>
      </div>
      <div class="header-actions">
        <nav class="app-nav" v-if="isAuthenticated">
          <RouterLink to="/">首页</RouterLink>
          <RouterLink v-if="canUpload" to="/upload">图像上传</RouterLink>
          <RouterLink to="/results">检测结果</RouterLink>
        </nav>
        <div class="account" v-if="isAuthenticated">
          <span class="account-name">{{ username }} · {{ roleLabel }}</span>
          <el-button type="text" @click="logout">退出</el-button>
        </div>
        <el-button v-else type="primary" @click="goLogin">登录</el-button>
      </div>
    </header>

    <main class="app-main">
      <router-view />
    </main>

    <footer class="app-footer">
      <small>后端基于 Rust + PostgreSQL · 缓存由 Redis 提供</small>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, RouterLink } from 'vue-router'
import { ElMessage } from 'element-plus'

import { useUserStore } from '@/stores/user'

const router = useRouter()
const store = useUserStore()

const isAuthenticated = computed(() => store.isAuthenticated.value)
const username = computed(() => store.user.value?.username ?? '未登录')
const roleLabel = computed(() => {
  const role = store.role.value
  switch (role.toLowerCase()) {
    case 'admin':
      return '管理员'
    case 'operator':
      return '操作员'
    case 'viewer':
      return '观察者'
    default:
      return role || '未知角色'
  }
})
const canUpload = computed(() => store.canUpload.value)

const logout = () => {
  store.clearAuth()
  ElMessage.success('已退出登录')
  router.push({ name: 'Login' })
}

const goLogin = () => {
  router.push({ name: 'Login' })
}
</script>

<style>
.app-shell {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f5f7f9;
  color: #1f2933;
  font-family: 'Helvetica Neue', Arial, sans-serif;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.75rem 2.5rem;
  background: #1abc9c;
  color: #fff;
  box-shadow: 0 6px 16px rgba(26, 188, 156, 0.25);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 1.5rem;
}

.app-header h1 {
  margin: 0;
  font-size: 1.9rem;
  font-weight: 600;
}

.subtitle {
  margin: 0.35rem 0 0;
  font-size: 0.95rem;
  opacity: 0.85;
}

.app-nav {
  display: flex;
  gap: 1.5rem;
}

.app-nav a {
  color: #fff;
  text-decoration: none;
  font-weight: 500;
  padding-bottom: 0.25rem;
  transition: opacity 0.2s ease, border-color 0.2s ease;
  border-bottom: 2px solid transparent;
}

.app-nav a:hover {
  opacity: 0.85;
}

.app-nav a.router-link-active {
  border-color: #fff;
}

.account {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.account-name {
  font-size: 0.95rem;
}

.app-main {
  flex: 1;
  padding: 2.5rem;
}

.app-footer {
  text-align: center;
  padding: 1rem;
  background: #ffffff;
  border-top: 1px solid #e4ebf3;
  color: #52606d;
}
</style>

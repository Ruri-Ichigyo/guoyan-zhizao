<template>
  <div class="app-shell">
    <header class="app-header">
      <div>
        <h1>果蔬智造核心平台</h1>
        <p class="subtitle">上传识别 · 智能分析 · 快速查询</p>
      </div>
      <nav class="app-nav">
        <RouterLink to="/">首页</RouterLink>
        <RouterLink to="/upload">图像上传</RouterLink>
        <RouterLink to="/results">检测结果</RouterLink>
      </nav>
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
import { RouterLink } from 'vue-router'
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

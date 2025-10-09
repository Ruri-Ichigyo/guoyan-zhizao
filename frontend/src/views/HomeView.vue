<template>
  <section class="home-hero">
    <div class="hero-intro">
      <h1>果蔬智造核心平台</h1>
      <p>
        轻量部署的果蔬检测方案：Rust 后端负责任务编排，FastAPI AI 服务完成识别分析，
        Redis 提供热点缓存，PostgreSQL 保存检测记录。
      </p>
      <div class="hero-actions">
        <el-button type="primary" size="large" @click="goTo('/upload')">
          <el-icon class="mr-1"><UploadFilled /></el-icon>
          上传并分析
        </el-button>
        <el-button size="large" @click="goTo('/results')">
          <el-icon class="mr-1"><List /></el-icon>
          查看结果
        </el-button>
      </div>
    </div>
    <div class="hero-highlights">
      <div class="highlight-card">
        <h3>核心能力</h3>
        <ul>
          <li>AI 推理服务直接返回分类与等级信息</li>
          <li>Redis 缓存结果，提高查询性能</li>
          <li>PostgreSQL 持久化上传记录和统计数据</li>
        </ul>
      </div>
      <div class="highlight-card">
        <h3>快速上手</h3>
        <ul>
          <li>本地或容器一键启动</li>
          <li>精简前端界面关注上传与查询</li>
          <li>RESTful API，易于二次集成</li>
        </ul>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { UploadFilled, List } from '@element-plus/icons-vue'

const router = useRouter()

const goTo = (path: string) => {
  router.push(path)
}
</script>

<style scoped>
.home-hero {
  display: grid;
  gap: 2rem;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  align-items: start;
}

.hero-intro {
  background: linear-gradient(135deg, #1abc9c 0%, #16a085 100%);
  color: #fff;
  padding: 2.5rem;
  border-radius: 20px;
  box-shadow: 0 24px 40px -24px rgba(26, 188, 156, 0.7);
}

.hero-intro h1 {
  margin: 0 0 1rem;
  font-size: 2.4rem;
  font-weight: 700;
}

.hero-intro p {
  margin: 0;
  line-height: 1.7;
  font-size: 1rem;
  opacity: 0.9;
}

.hero-actions {
  margin-top: 1.5rem;
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.hero-highlights {
  display: grid;
  gap: 1rem;
}

.highlight-card {
  background: #fff;
  border-radius: 16px;
  padding: 1.75rem;
  box-shadow: 0 18px 32px -20px rgba(15, 23, 42, 0.2);
}

.highlight-card h3 {
  margin: 0 0 1rem;
  font-size: 1.25rem;
  color: #0f172a;
}

.highlight-card ul {
  margin: 0;
  padding-left: 1.2rem;
  color: #475569;
  line-height: 1.8;
  font-size: 0.95rem;
}
</style>

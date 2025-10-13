<template>
  <div
    class="home-container flex flex-col lg:flex-row items-center lg:items-start justify-center gap-12 lg:gap-16 p-8"
  >
    <section class="hero-section text-center lg:text-left max-w-xl animate-fade-in">
      <h1 class="system-title text-4xl lg:text-5xl font-extrabold text-green-700 mb-4">
        果蔬智能分拣系统
      </h1>
      <h2 class="welcome-title text-3xl lg:text-4xl font-bold text-gray-800 mb-4">
        欢迎来到果蔬智能分拣系统
      </h2>
      <p class="system-description text-lg lg:text-xl text-gray-700 mb-8 animate-slide-up">
        通过先进的图像识别和自动化技术，实现高效、精准的果蔬分拣。
      </p>
    </section>

    <section class="features-section w-full max-w-5xl">
      <div class="features-grid grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
        <!-- 第一行：3个卡片 -->
        <div class="feature-card-custom animate-zoom-in" @click="goTo('/upload')">
          <div class="card-content">
            <el-icon class="card-icon"><Upload /></el-icon>
            <h2 class="card-title">图像上传</h2>
            <p class="card-description">上传图像进行分拣分析</p>
          </div>
        </div>

        <div class="feature-card-custom animate-zoom-in delay-100" @click="goTo('/camera')">
          <div class="card-content">
            <el-icon class="card-icon"><Camera /></el-icon>
            <h2 class="card-title">实时摄像</h2>
            <p class="card-description">实时监控分拣过程</p>
          </div>
        </div>

        <div class="feature-card-custom animate-zoom-in delay-200" @click="goTo('/dashboard')">
          <div class="card-content">
            <el-icon class="card-icon"><DataAnalysis /></el-icon>
            <h2 class="card-title">数据大屏</h2>
            <p class="card-description">查看分拣统计数据</p>
          </div>
        </div>

        <!-- 第二行：3个卡片 -->
        <div class="feature-card-custom animate-zoom-in delay-300" @click="goTo('/results')">
          <div class="card-content">
            <el-icon class="card-icon"><List /></el-icon>
            <h2 class="card-title">结果查看</h2>
            <p class="card-description">查看历史分拣结果</p>
          </div>
        </div>

        <div class="feature-card-custom animate-zoom-in delay-400" @click="goTo('/detections')">
          <div class="card-content">
            <el-icon class="card-icon"><VideoCamera /></el-icon>
            <h2 class="card-title">识别记录</h2>
            <p class="card-description">查看实时监控识别记录</p>
          </div>
        </div>

        <div class="feature-card-custom animate-zoom-in delay-500" @click="goTo('/sort')">
          <div class="card-content">
            <el-icon class="card-icon"><Operation /></el-icon>
            <h2 class="card-title">分拣控制</h2>
            <p class="card-description">控制分拣系统运行</p>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { Upload, Camera, DataAnalysis, List, VideoCamera, Operation } from '@element-plus/icons-vue' // Import necessary icons

const router = useRouter()

const goTo = (path: string) => {
  router.push(path)
}
</script>

<style scoped>
.home-container {
  /* No background here, handled by App.vue */
  min-height: calc(100vh - 40px); /* Adjust for main-content-background margin */
  position: relative;
  z-index: 1; /* Ensure content is above background elements */
}

.hero-section {
  width: 100%;
}

.features-section {
  width: 100%;
}

.features-grid {
  margin: 0 auto;
}

.system-title {
  font-family: var(--font-family-base); /* Use new font family */
  color: var(--color-primary-dark); /* Use dark primary color */
  text-shadow: none; /* Remove text shadow for cleaner look */
}

.welcome-title {
  font-family: var(--font-family-base);
  color: var(--color-text-primary); /* Use primary text color */
}

.system-description {
  font-family: var(--font-family-base);
  color: var(--color-text-secondary); /* Use secondary text color */
  line-height: 1.6;
}

.feature-card-custom {
  cursor: pointer;
  transition: transform 0.3s ease, box-shadow 0.3s ease;
  border-radius: 12px; /* Slightly less rounded corners */
  background-color: var(--color-white); /* Solid white background */
  backdrop-filter: none; /* Remove backdrop filter */
  border: 1px solid var(--color-border); /* Use new border color */
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08); /* More subtle shadow */
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px; /* Ensure consistent height */
  position: relative;
  overflow: hidden;
}

.feature-card-custom::before {
  content: none; /* Remove pseudo-element */
}

.feature-card-custom:hover {
  transform: translateY(-5px) scale(1.01); /* More subtle hover effect */
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.12);
}

.card-content {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.card-icon {
  width: 60px; /* Adjust icon size */
  height: 60px;
  margin-bottom: 15px;
  color: var(--color-primary); /* Use primary color for icons */
  filter: none; /* Remove drop shadow */
}

.card-title {
  font-size: 24px; /* Slightly smaller title */
  font-weight: 600; /* Medium bold */
  color: var(--color-text-primary); /* Use primary text color */
  margin-bottom: 8px;
}

.card-description {
  font-size: 15px; /* Slightly smaller description */
  color: var(--color-text-secondary); /* Use secondary text color */
}

/* Animations */
@keyframes fade-in {
  from { opacity: 0; transform: translateY(-20px); }
  to { opacity: 1; transform: translateY(0); }
}
.animate-fade-in {
  animation: fade-in 0.8s ease-out forwards;
}

.animate-fade-in.delay-100 {
  animation-delay: 0.3s;
}

@keyframes slide-up {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}
.animate-slide-up {
  animation: slide-up 0.8s ease-out forwards;
  animation-delay: 0.5s;
}

@keyframes zoom-in {
  from { opacity: 0; transform: scale(0.9); }
  to { opacity: 1; transform: scale(1); }
}
.animate-zoom-in {
  animation: zoom-in 0.6s ease-out forwards;
}
.animate-zoom-in.delay-100 { animation-delay: 0.8s; }
.animate-zoom-in.delay-200 { animation-delay: 1.0s; }
.animate-zoom-in.delay-300 { animation-delay: 1.2s; }
.animate-zoom-in.delay-400 { animation-delay: 1.4s; }
.animate-zoom-in.delay-500 { animation-delay: 1.6s; }

@keyframes rotate {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>

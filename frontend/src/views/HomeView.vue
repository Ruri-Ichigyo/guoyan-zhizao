<template>
  <div class="home-container flex flex-col items-center justify-center p-8">
    <h1 class="system-title text-5xl font-extrabold text-green-700 mb-6 animate-fade-in">
      果蔬智能分拣系统
    </h1>
    <h2 class="welcome-title text-4xl font-bold text-gray-800 mb-4 animate-fade-in delay-100">
      欢迎来到果蔬智能分拣系统
    </h2>
    <p class="system-description text-xl text-gray-700 mb-10 text-center max-w-2xl animate-slide-up">
      通过先进的图像识别和自动化技术，实现高效、精准的果蔬分拣。
    </p>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 w-full max-w-4xl">
      <div class="feature-card-custom animate-zoom-in" @click="goTo('/upload')">
        <div class="card-content">
          <img src="/images/upload_icon.png" alt="图像上传" class="card-icon" /> <!-- Placeholder -->
          <h2 class="card-title">图像上传</h2>
          <p class="card-description">上传图像进行分拣分析</p>
        </div>
      </div>

      <div class="feature-card-custom animate-zoom-in delay-100" @click="goTo('/camera')">
        <div class="card-content">
          <img src="/images/camera_icon.png" alt="实时摄像" class="card-icon" /> <!-- Placeholder -->
          <h2 class="card-title">实时摄像</h2>
          <p class="card-description">实时监控分拣过程</p>
        </div>
      </div>

      <div class="feature-card-custom animate-zoom-in delay-200" @click="goTo('/dashboard')">
        <div class="card-content">
          <img src="/images/dashboard_icon.png" alt="数据大屏" class="card-icon" /> <!-- Placeholder -->
          <h2 class="card-title">数据大屏</h2>
          <p class="card-description">查看分拣统计数据</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { Upload, Camera, DataAnalysis, List, Operation } from '@element-plus/icons-vue'

const router = useRouter()

const goTo = (path: string) => {
  router.push(path)
}
</script>

<style scoped>
.home-container {
  /* No background here, handled by App.vue */
  min-height: calc(100vh - 40px); /* Adjust for main-content-background margin */
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
  z-index: 1; /* Ensure content is above background elements */
}

.system-title {
  font-family: 'Arial Black', Gadget, sans-serif; /* Example font, adjust as needed */
  color: #336600; /* Dark green */
  text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
  margin-bottom: 20px;
}

.welcome-title {
  color: #4CAF50; /* Green */
  margin-bottom: 10px;
}

.system-description {
  color: #555;
  line-height: 1.6;
}

.feature-card-custom {
  cursor: pointer;
  transition: transform 0.3s ease, box-shadow 0.3s ease;
  border-radius: 20px; /* More rounded corners */
  background-color: rgba(255, 255, 255, 0.7); /* Semi-transparent white */
  backdrop-filter: blur(10px); /* Frosted glass effect */
  border: 2px solid rgba(255, 255, 255, 0.5); /* Lighter border */
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.1);
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px; /* Ensure consistent height */
  position: relative;
  overflow: hidden; /* For internal patterns */
}

.feature-card-custom::before {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(circle at center, rgba(255, 255, 255, 0.1) 0%, rgba(255, 255, 255, 0) 70%);
  animation: rotate 10s linear infinite;
  z-index: 0;
}

.feature-card-custom:hover {
  transform: translateY(-10px) scale(1.02);
  box-shadow: 0 15px 40px rgba(0, 0, 0, 0.2);
}

.card-content {
  position: relative;
  z-index: 1; /* Ensure content is above pseudo-element */
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.card-icon {
  width: 80px; /* Adjust icon size */
  height: 80px;
  margin-bottom: 15px;
  filter: drop-shadow(2px 2px 4px rgba(0, 0, 0, 0.2)); /* Add shadow to icons */
}

.card-title {
  font-size: 28px; /* Larger title */
  font-weight: bold;
  color: #333;
  margin-bottom: 8px;
}

.card-description {
  font-size: 16px;
  color: #666;
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
.animate-zoom-in.delay-200 { animation-delay: 1.1s; }

@keyframes rotate {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>

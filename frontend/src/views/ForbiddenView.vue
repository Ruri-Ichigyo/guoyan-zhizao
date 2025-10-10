<template>
  <section class="forbidden">
    <el-result icon="warning" title="没有访问权限" sub-title="请联系管理员获取更高的权限等级">
      <template #extra>
        <el-button type="primary" @click="goHome">返回首页</el-button>
      </template>
    </el-result>
  </section>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

const goHome = () => {
  router.push('/')
}
</script>

<style scoped>
.forbidden {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 60vh;
}
</style>

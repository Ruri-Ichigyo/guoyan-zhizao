<template>
  <section class="login-wrapper">
    <el-card class="login-card" shadow="hover">
      <h2 class="title">登录果蔬智造平台</h2>
      <p class="subtitle">请输入账号和密码以继续访问核心功能</p>

      <el-form :model="form" :rules="rules" ref="formRef" label-position="top">
        <el-form-item label="用户名" prop="username">
          <el-input v-model="form.username" placeholder="输入用户名" autocomplete="username" />
        </el-form-item>
        <el-form-item label="密码" prop="password">
          <el-input
            v-model="form.password"
            type="password"
            placeholder="输入密码"
            autocomplete="current-password"
            show-password
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :loading="submitting" @click="handleSubmit" round block>
            登录
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </section>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { FormInstance, FormRules } from 'element-plus'
import { ElMessage } from 'element-plus'

import { authApi } from '@/api/auth'
import { useUserStore } from '@/stores/user'

const router = useRouter()
const route = useRoute()
const userStore = useUserStore()

const form = reactive({
  username: '',
  password: ''
})

const submitting = ref(false)
const formRef = ref<FormInstance>()

const rules: FormRules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  password: [{ required: true, message: '请输入密码', trigger: 'blur' }]
}

const redirectAfterLogin = () => {
  const redirect = (route.query.redirect as string | undefined) ?? '/'
  router.replace(redirect)
}

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid) => {
    if (!valid) return
    submitting.value = true
    try {
      const response = await authApi.login({ ...form })
      userStore.setAuth(response.token, response.expires_at, response.user)
      ElMessage.success('登录成功')
      redirectAfterLogin()
    } catch (error) {
      console.error(error)
      ElMessage.error('登录失败，请检查用户名或密码')
    } finally {
      submitting.value = false
    }
  })
}

onMounted(() => {
  if (userStore.ensureSession()) {
    redirectAfterLogin()
  }
})
</script>

<style scoped>
.login-wrapper {
  min-height: 70vh;
  display: flex;
  justify-content: center;
  align-items: center;
}

.login-card {
  width: min(420px, 100%);
  border-radius: 16px;
}

.title {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.subtitle {
  margin: 0.5rem 0 1.5rem;
  color: #64748b;
}
</style>

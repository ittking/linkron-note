<script setup>
import { ref, computed } from 'vue'
import { Mail, Lock, LogIn, Eye, EyeOff, Loader2 } from 'lucide-vue-next'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'

const props = defineProps({
  loading: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['submit', 'register'])

// 表单数据
const email = ref('')
const password = ref('')
const showPassword = ref(false)

// 错误信息
const emailError = ref('')
const passwordError = ref('')

// 验证邮箱格式
function validateEmail(email) {
  const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return re.test(email)
}

// 验证表单
function validateForm() {
  emailError.value = ''
  passwordError.value = ''
  let isValid = true

  if (!email.value.trim()) {
    emailError.value = '请输入邮箱'
    isValid = false
  } else if (!validateEmail(email.value)) {
    emailError.value = '请输入有效的邮箱地址'
    isValid = false
  }

  if (!password.value) {
    passwordError.value = '请输入密码'
    isValid = false
  } else if (password.value.length < 6) {
    passwordError.value = '密码至少6位'
    isValid = false
  }

  return isValid
}

// 提交登录
function handleSubmit() {
  if (validateForm()) {
    emit('submit', {
      email: email.value,
      password: password.value
    })
  }
}

// 跳转注册
function handleRegister() {
  emit('register')
}

// 切换密码显示
function togglePassword() {
  showPassword.value = !showPassword.value
}

// 回车登录
function handleKeyup(e) {
  if (e.key === 'Enter') {
    handleSubmit()
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-base-200 p-4">
    <div class="w-full max-w-md">
      <!-- Logo 和标题 -->
      <div class="text-center mb-8">
        <div class="w-20 h-20 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-primary to-secondary flex items-center justify-center shadow-lg">
          <span class="text-4xl font-bold text-base-content">L</span>
        </div>
        <h1 class="text-2xl font-bold text-base-content">LINKRON</h1>
        <p class="text-sm text-base-content/60 mt-1">极简笔记，随时随记</p>
      </div>

      <!-- 登录表单 -->
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body p-6">
          <h2 class="card-title text-lg font-medium mb-6">登录账号</h2>

          <div class="space-y-4">
            <!-- 邮箱输入 -->
            <div class="form-control">
              <label class="label">
                <span class="label-text text-sm">邮箱</span>
              </label>
              <div class="relative">
                <Mail :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
                <Input
                  v-model="email"
                  type="email"
                  placeholder="请输入邮箱"
                  :error="!!emailError"
                  @keyup="handleKeyup"
                  class="pl-10"
                />
              </div>
              <label v-if="emailError" class="label">
                <span class="label-text-alt text-error">{{ emailError }}</span>
              </label>
            </div>

            <!-- 密码输入 -->
            <div class="form-control">
              <label class="label">
                <span class="label-text text-sm">密码</span>
              </label>
              <div class="relative">
                <Lock :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
                <input
                  v-model="password"
                  :type="showPassword ? 'text' : 'password'"
                  placeholder="请输入密码"
                  :class="[
                    'w-full rounded-lg border transition-all duration-200 outline-none pl-10 pr-10',
                    'focus:ring-2 focus:ring-offset-2 focus:ring-offset-base-100',
                    'placeholder:text-base-content/50',
                    passwordError
                      ? 'border-error focus:border-error focus:ring-error'
                      : 'border-base-300 focus:border-primary focus:ring-primary',
                    'bg-base-100 text-base-content',
                    'px-3 py-2 text-sm min-h-[38px]'
                  ]"
                  @keyup="handleKeyup"
                />
                <button
                  type="button"
                  @click="togglePassword"
                  class="absolute right-3 top-1/2 -translate-y-1/2 text-base-content/40 hover:text-base-content/60 transition-colors"
                >
                  <EyeOff v-if="showPassword" :size="16" />
                  <Eye v-else :size="16" />
                </button>
              </div>
              <label v-if="passwordError" class="label">
                <span class="label-text-alt text-error">{{ passwordError }}</span>
              </label>
            </div>

            <!-- 登录按钮 -->
            <Button
              variant="primary"
              size="md"
              block
              :loading="loading"
              @click="handleSubmit"
              class="mt-6"
            >
              <template v-if="!loading">
                <LogIn :size="16" />
                登录
              </template>
              <template v-else>
                <Loader2 :size="16" class="animate-spin" />
                登录中...
              </template>
            </Button>

            <!-- 注册链接 -->
            <div class="text-center mt-4">
              <span class="text-sm text-base-content/60">还没有账号？</span>
              <button
                type="button"
                @click="handleRegister"
                class="text-sm text-primary hover:text-primary/80 transition-colors ml-1"
              >
                立即注册
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 底部信息 -->
      <div class="text-center mt-6 text-xs text-base-content/40">
        <p>登录即表示同意《用户协议》和《隐私政策》</p>
      </div>
    </div>
  </div>
</template>

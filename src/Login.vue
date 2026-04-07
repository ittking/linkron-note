<script setup>
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from './store/authStore'
import { BookOpen } from 'lucide-vue-next'
import WindowControls from './components/ui/WindowControls.vue'
import { Loader2, QrCode, Clock, CheckCircle2, XCircle, RefreshCw } from 'lucide-vue-next'

const router = useRouter()
const authStore = useAuthStore()

// 状态
const countdown = ref(60)
const showQRCode = ref(false)
const canGetQRCode = ref(true)
const timer = ref(null)
const logoUrl = ref('/src/assets/128x128.png')
const appName = ref('LINKRON')

// 从 authStore 解构响应式数据
const { qrCodeData, authStatus, isLoading, isLoggedIn } = authStore

// 状态显示文本
const statusText = computed(() => {
  switch (authStatus.value) {
    case 'idle':
      return ''
    case 'pending':
      return '请使用微信扫描二维码登录'
    case 'authorized':
      return '授权成功，正在跳转...'
    case 'expired':
      return '二维码已过期，请重新获取'
    case 'error':
      return '获取二维码失败，请重试'
    default:
      return ''
  }
})

// 状态图标
const statusIcon = computed(() => {
  switch (authStatus.value) {
    case 'pending':
      return Loader2
    case 'authorized':
      return CheckCircle2
    case 'expired':
      return XCircle
    case 'error':
      return XCircle
    default:
      return null
  }
})

// 状态颜色
const statusColor = computed(() => {
  switch (authStatus.value) {
    case 'pending':
      return 'text-info'
    case 'authorized':
      return 'text-success'
    case 'expired':
    case 'error':
      return 'text-error'
    default:
      return ''
  }
})

// 是否显示加载中
const showLoading = computed(() => isLoading.value || authStatus.value === 'pending')

// 获取二维码
async function handleGetQRCode() {
  if (!canGetQRCode.value) return

  try {
    await authStore.initAuth()
    showQRCode.value = true
    startCountdown()
  } catch (error) {
    console.error('获取二维码失败:', error)
  }
}

// 开始倒计时
function startCountdown() {
  canGetQRCode.value = false
  countdown.value = 60

  timer.value = setInterval(() => {
    countdown.value--

    if (countdown.value <= 0) {
      stopCountdown()
      // 如果还在等待授权，标记为过期
      if (authStatus.value === 'pending') {
        authStore.stopPolling()
        authStore.setExpired()
      }
    }
  }, 1000)
}

// 停止倒计时
function stopCountdown() {
  if (timer.value) {
    clearInterval(timer.value)
    timer.value = null
  }
  canGetQRCode.value = true
}

// 重新获取
function handleRefresh() {
  authStore.resetAuth()
  showQRCode.value = false
  stopCountdown()
}

// 监听登录状态变化
watch(isLoggedIn, (newValue) => {
  if (newValue) {
    // 登录成功，跳转到主页
    setTimeout(() => {
      router.push('/note')
    }, 500)
  }
})

// 组件卸载时清理
onBeforeUnmount(() => {
  stopCountdown()
  authStore.stopPolling()
})
</script>

<template>
  <div class="h-screen flex flex-col bg-base-200">
    <!-- 顶部控制栏 -->
    <div data-tauri-drag-region class="h-9 flex items-center justify-between px-3 flex-shrink-0">
      <!-- 左侧：应用图标和名称 -->
      <div class="flex items-center gap-2">
        <BookOpen :size="16" class="text-primary" data-tauri-drag-region />
        <span class="text-sm font-medium text-base-content" data-tauri-drag-region>LINKRON</span>
      </div>
      <!-- 右侧：窗口控制 -->
      <WindowControls />
    </div>

    <!-- 登录内容 -->
    <div class="flex-1 flex items-center justify-center p-4">
      <div class="w-full max-w-sm">
      <!-- Logo 和标题 -->
      <div class="text-center mb-8">
        <div class="w-20 h-20 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-primary to-secondary flex items-center justify-center shadow-lg overflow-hidden">
          <img :src="logoUrl" :alt="appName" class="w-full h-full object-cover" />
        </div>
        <h1 class="text-2xl font-bold text-base-content">{{ appName }}</h1>
        <p class="text-sm text-base-content/60 mt-1">极简笔记，随时随记</p>
      </div>

      <!-- 二维码区域 -->
      <div v-if="showQRCode" class="card bg-base-100 shadow-xl">
        <div class="card-body p-6">
          <h2 class="card-title text-lg font-medium mb-4 justify-center">微信扫码登录</h2>

          <!-- 二维码显示 -->
          <div class="flex flex-col items-center">
            <!-- 二维码图片 -->
            <div v-if="qrCodeData?.base64" class="relative mb-4">
              <img
                :src="`data:${qrCodeData.contentType};base64,${qrCodeData.base64}`"
                alt="登录二维码"
                class="w-48 h-48 rounded-lg border-2 border-base-300"
              />
              <!-- 状态遮罩 -->
              <div v-if="authStatus === 'expired'" class="absolute inset-0 bg-base-100/80 rounded-lg flex items-center justify-center">
                <div class="text-center">
                  <XCircle :size="32" class="mx-auto text-error mb-2" />
                  <p class="text-sm text-error">已过期</p>
                </div>
              </div>
            </div>

            <!-- 加载状态 -->
            <div v-else-if="showLoading" class="w-48 h-48 rounded-lg border-2 border-base-300 flex items-center justify-center">
              <Loader2 :size="32" class="animate-spin text-primary" />
            </div>

            <!-- 状态文本 -->
            <div v-if="statusText" :class="['flex items-center gap-2 text-sm', statusColor]">
              <component v-if="statusIcon" :is="statusIcon" :size="16" :class="{ 'animate-spin': authStatus === 'pending' }" />
              <span>{{ statusText }}</span>
            </div>

            <!-- 倒计时 -->
            <div v-if="countdown > 0 && authStatus === 'pending'" class="flex items-center gap-1.5 text-xs text-base-content/60 mt-2">
              <Clock :size="12" />
              <span>有效期: {{ countdown }}秒</span>
            </div>

            <!-- 重新获取按钮 -->
            <button
              v-if="authStatus === 'expired' || authStatus === 'error'"
              @click="handleRefresh"
              class="mt-4 btn btn-outline btn-sm gap-2"
            >
              <RefreshCw :size="14" />
              重新获取
            </button>
          </div>
        </div>
      </div>

      <!-- 微信授权登录按钮 -->
      <button
        v-else
        @click="handleGetQRCode"
        :disabled="!canGetQRCode"
        class="btn btn-primary w-full gap-2"
      >
        <QrCode :size="18" />
        微信授权登录
      </button>

        <!-- 底部信息 -->
        <div class="text-center mt-6 text-xs text-base-content/40">
          <p>登录即表示同意《用户协议》和《隐私政策》</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 无额外样式，使用 DaisyUI */
</style>

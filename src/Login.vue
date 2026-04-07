<script setup>
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from './store/authStore'
import { BookOpen } from 'lucide-vue-next'
import WindowControls from './components/ui/WindowControls.vue'
import { QrCode, Clock, RefreshCw, Sparkles } from 'lucide-vue-next'
import { generateAuthCode, getQRCode, pollAuthStatus, getAppKey } from '@/api/auth'

const router = useRouter()
const authStore = useAuthStore()

// 本地状态
const countdown = ref(60)
const showQRCode = ref(false)
const canRefresh = ref(false)  // 重新获取按钮是否可用
const timer = ref(null)
const logoUrl = ref('/src/assets/128x128.png')
const appName = ref('LINKRON')

// 授权状态：idle, loading, pending, error, success, expired
const localAuthStatus = ref('idle')

// 二维码数据
const qrCodeData = ref(null)

// 授权码（本地存储）
const authCode = ref('')

// 从 authStore 获取登录状态
const isLoggedIn = computed(() => authStore.isLoggedIn.value)

// 是否显示加载中
const showLoading = computed(() => localAuthStatus.value === 'loading')

// 获取二维码
async function handleGetQRCode() {
  localAuthStatus.value = 'loading'
  qrCodeData.value = null

  try {
    // 检查 AppKey 配置
    const appKey = getAppKey()
    if (!appKey) {
      localAuthStatus.value = 'error'
      return
    }

    // 生成授权码
    authCode.value = generateAuthCode()

    // 获取小程序码
    const response = await getQRCode(authCode.value, appKey)

    if (response.success) {
      qrCodeData.value = response.data
      localAuthStatus.value = 'pending'
      showQRCode.value = true

      // 开始60秒倒计时（控制重新获取按钮）
      startCountdown()

      // 开始轮询检查状态
      startPolling()
    } else {
      localAuthStatus.value = 'error'
    }
  } catch (error) {
    console.error('获取二维码失败:', error)
    localAuthStatus.value = 'error'
  }
}

// 开始倒计时
function startCountdown() {
  canRefresh.value = false
  countdown.value = 60

  timer.value = setInterval(() => {
    countdown.value--

    if (countdown.value <= 0) {
      stopCountdown()
      // 二维码过期，停止轮询
      localAuthStatus.value = 'expired'
      stopPolling()
      canRefresh.value = true  // 可以重新获取
    }
  }, 1000)
}

// 停止倒计时
function stopCountdown() {
  if (timer.value) {
    clearInterval(timer.value)
    timer.value = null
  }
}

// 轮询定时器
let pollingTimer = null

// 开始轮询
function startPolling() {
  stopPolling()

  // 立即检查一次
  checkAuthStatus()

  // 每3秒检查一次
  pollingTimer = setInterval(() => {
    checkAuthStatus()
  }, 5000)
}

// 停止轮询
function stopPolling() {
  if (pollingTimer) {
    clearInterval(pollingTimer)
    pollingTimer = null
  }
}

// 检查授权状态
async function checkAuthStatus() {
  if (localAuthStatus.value !== 'pending') return

  try {
    if (!authCode.value) return

    const response = await pollAuthStatus(authCode.value)

    if (response.success && response.data) {
      const { status, user: userData, token: tokenValue } = response.data

      switch (status) {
        case 'pending':
          // 继续等待
          break
        case 'authorized':
          // 授权成功
          localAuthStatus.value = 'success'
          authStore.saveAuth(tokenValue, userData)
          stopPolling()
          stopCountdown()
          canRefresh.value = true
          break
        default:
          // 其他状态视为错误，停止轮询
          localAuthStatus.value = 'error'
          stopPolling()
          stopCountdown()
          canRefresh.value = true
      }
    }
  } catch (error) {
    console.error('检查授权状态失败:', error)
  }
}

// 重新获取二维码
async function handleRefresh() {
  if (!canRefresh.value) return

  // 停止当前的轮询和倒计时
  stopCountdown()
  stopPolling()

  // 重新获取二维码
  await handleGetQRCode()
}

// 监听登录状态变化
watch(isLoggedIn, (newValue) => {
  if (newValue) {
    setTimeout(() => {
      router.push('/note')
    }, 500)
  }
})

// 组件卸载时清理
onBeforeUnmount(() => {
  stopCountdown()
  stopPolling()
})
</script>

<template>
  <div class="h-screen flex flex-col bg-base-100">
    <!-- 顶部控制栏 -->
    <div data-tauri-drag-region
      class="h-9 flex items-center justify-between px-3 flex-shrink-0 border-b border-base-200">
      <div class="flex items-center gap-2">
        <BookOpen :size="16" class="text-primary" data-tauri-drag-region />
        <span class="text-sm font-medium text-base-content" data-tauri-drag-region>LINKRON</span>
      </div>
      <WindowControls />
    </div>

    <!-- 登录内容 -->
    <div class="flex-1 flex items-center justify-center p-6 relative overflow-hidden">
      <!-- 背景装饰 -->
      <div class="absolute inset-0 overflow-hidden pointer-events-none">
        <div class="absolute -top-40 -right-40 w-80 h-80 bg-primary/5 rounded-full blur-3xl"></div>
        <div class="absolute -bottom-40 -left-40 w-80 h-80 bg-secondary/5 rounded-full blur-3xl"></div>
      </div>

      <div class="w-full max-w-sm relative z-10">
        <!-- Logo 和标题 -->
        <div class="text-center mb-10">
          <div
            class="w-20 h-20 mx-auto mb-5 rounded-2xl bg-gradient-to-br from-primary/10 to-secondary/10 flex items-center justify-center ring-1 ring-base-200/50 shadow-lg shadow-primary/5">
            <img :src="logoUrl" :alt="appName" class="w-12 h-12 object-contain" />
          </div>
          <h1 class="text-3xl font-bold text-base-content tracking-tight mb-2">{{ appName }}</h1>
          <p class="text-sm text-base-content/60">极简笔记，随时随记</p>
        </div>

        <!-- 二维码区域 -->
        <transition mode="out-in" enter-active-class="transition-opacity duration-300 ease-out"
          enter-from-class="opacity-0" enter-to-class="opacity-100"
          leave-active-class="transition-opacity duration-300 ease-in" leave-from-class="opacity-100"
          leave-to-class="opacity-0">
          <div v-if="showQRCode" class="p-8">
            <h2
              class="text-base font-semibold text-base-content mb-6 text-center flex items-center justify-center gap-2">
              <Sparkles :size="18" class="text-primary" />
              微信扫码登录
            </h2>

            <div class="flex flex-col items-center">
              <!-- 二维码图片 -->
              <div v-if="qrCodeData?.base64" class="relative">
                <div
                  class="w-52 h-52 rounded-2xl p-3 bg-gradient-to-br from-base-50 to-base-100 ring-1 ring-base-200/50">
                  <img :src="`data:${qrCodeData.contentType};base64,${qrCodeData.base64}`" alt="登录二维码"
                    class="w-full h-full rounded-xl object-contain ring-1 ring-base-200/30" />
                </div>

                <!-- 扫描动画 -->
                <div v-if="localAuthStatus === 'pending'"
                  class="absolute inset-0 rounded-2xl overflow-hidden pointer-events-none">
                  <div class="scan-line"></div>
                </div>

                <!-- 过期遮罩 -->
                <div v-if="localAuthStatus === 'expired'"
                  class="absolute inset-0 rounded-2xl bg-base-100/80 backdrop-blur-sm flex flex-col items-center justify-center gap-3">
                  <Clock :size="32" class="text-error" />
                  <span class="text-sm font-medium text-error">二维码已过期</span>
                </div>
              </div>

              <!-- 加载状态 -->
              <div v-else-if="showLoading"
                class="w-52 h-52 rounded-2xl bg-base-100 ring-1 ring-base-200/50 flex items-center justify-center">
                <span class="loading loading-spinner loading-lg text-primary"></span>
              </div>

              <!-- 重新获取按钮 - 倒计时期间禁用 -->
              <button @click="handleRefresh" :disabled="!canRefresh" class="mt-6 btn btn-outline btn-sm gap-2"
                :class="canRefresh ? 'border-base-200 hover:border-primary hover:text-primary' : 'border-base-300 text-base-content/40 cursor-not-allowed'">
                <RefreshCw :size="14" />
                重新获取二维码
                <span v-if="!canRefresh && countdown > 0" class="text-xs">({{ countdown }}s)</span>
              </button>
            </div>
          </div>
        </transition>

        <!-- 微信授权登录按钮 -->
        <transition mode="out-in" enter-active-class="transition-opacity duration-300 ease-out"
          enter-from-class="opacity-0" enter-to-class="opacity-100"
          leave-active-class="transition-opacity duration-300 ease-in" leave-from-class="opacity-100"
          leave-to-class="opacity-0">
          <div v-if="!showQRCode" class="space-y-4">
            <button @click="handleGetQRCode"
              class="btn btn-primary w-full gap-2 h-12 text-base font-medium shadow-lg shadow-primary/25 hover:shadow-primary/30">
              <QrCode :size="20" />
              微信授权登录
            </button>

            <div class="text-center">
              <p class="text-xs text-base-content/40">登录即表示同意《用户协议》和《隐私政策》</p>
            </div>
          </div>
        </transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 扫描线动画 */
.scan-line {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, hsl(var(--p)), transparent);
  animation: scan 2s ease-in-out infinite;
}

@keyframes scan {
  0% {
    top: 0;
    opacity: 0;
  }

  10% {
    opacity: 1;
  }

  90% {
    opacity: 1;
  }

  100% {
    top: 100%;
    opacity: 0;
  }
}
</style>

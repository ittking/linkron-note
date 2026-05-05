<script setup>
import { ref, onMounted } from 'vue'
import { Info, MessageCircle, Heart, Download, RefreshCw, CheckCircle, AlertCircle, ExternalLink } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import { getVersion } from '@tauri-apps/api/app'
import { useAutoUpdater } from '@/composables/useAutoUpdater'
import wechatQR from '@/assets/weixin_gz.jpg'
import appLogo from '@/assets/128x128.png'

const RELEASES_URL = 'https://github.com/ittking/linkron-note/releases'

const appVersion = ref('')
const buildDate = '2025-02-16'
const updateStatus = ref('idle') // idle, checking, available, error, no-update
const updateMessage = ref('')
const latestVersion = ref('')

const { manualCheck } = useAutoUpdater(appVersion)

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch (e) {
    console.error('Failed to get version:', e)
  }
})

async function checkForUpdate() {
  updateStatus.value = 'checking'
  updateMessage.value = '正在检查更新...'

  try {
    const result = await manualCheck(appVersion.value)

    if (result.updateAvailable.value) {
      latestVersion.value = result.latestVersion.value
      updateStatus.value = 'available'
      updateMessage.value = `发现新版本 v${latestVersion.value}`
    } else {
      updateStatus.value = 'no-update'
      updateMessage.value = '已是最新版本'
      latestVersion.value = ''
      setTimeout(() => {
        updateStatus.value = 'idle'
        updateMessage.value = ''
      }, 3000)
    }
  } catch (error) {
    updateStatus.value = 'error'
    updateMessage.value = `检查更新失败: ${error?.message || String(error)}`
    console.error('Update check failed:', error)
  }
}

function openReleases() {
  window.open(RELEASES_URL, '_blank')
}
</script>

<template>
  <div class="space-y-4">
    <!-- 版本信息 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-4">
          <Info :size="16" />
          关于 LINKRON
        </h2>

        <div class="flex items-center gap-4 mb-4">
          <div class="w-16 h-16 rounded-xl overflow-hidden shadow-md">
            <img :src="appLogo" alt="LINKRON Logo" class="w-full h-full object-cover" />
          </div>
          <div>
            <h3 class="text-lg font-semibold">LINKRON</h3>
            <p class="text-sm text-base-content/60">极简笔记，随时随记</p>
            <div class="flex items-center gap-2 mt-1">
              <span class="text-xs text-base-content/40">版本 {{ appVersion }}</span>
              <span class="text-xs text-base-content/30">|</span>
              <span class="text-xs text-base-content/40">{{ buildDate }}</span>
            </div>
          </div>
        </div>

        <p class="text-xs text-base-content/60 leading-relaxed">
          LINKRON 是一款极简风格的跨平台桌面应用，集笔记、待办、日历于一体，随时随记，简约高效。
        </p>
      </div>
    </div>

    <!-- 更新设置 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between mb-4">
          <h2 class="card-title text-sm font-medium m-0">
            <RefreshCw :size="16" />
            版本更新
          </h2>
          <div class="flex items-center gap-2">
            <span
              v-if="updateStatus === 'available'"
              class="w-2 h-2 rounded-full bg-error animate-pulse"
              title="有新版本"
            ></span>
            <Button
              v-if="updateStatus !== 'checking'"
              @click="checkForUpdate"
              variant="ghost"
              size="sm"
            >
              <RefreshCw :size="12" />
              检查更新
            </Button>
          </div>
        </div>

        <!-- 当前版本 -->
        <div class="flex items-center justify-between p-3 rounded-lg bg-base-100">
          <span class="text-sm">当前版本</span>
          <span class="text-sm font-medium">{{ appVersion }}</span>
        </div>

        <!-- 最新版本 -->
        <div v-if="latestVersion" class="flex items-center justify-between p-3 rounded-lg bg-base-100 mt-2">
          <span class="text-sm">最新版本</span>
          <span class="text-sm font-medium text-success">v{{ latestVersion }}</span>
        </div>

        <!-- 更新状态消息 -->
        <div v-if="updateMessage" class="mt-3">
          <div
            :class="[
              'flex items-center gap-2 p-2 rounded-lg text-xs',
              updateStatus === 'error' ? 'bg-error/10 text-error' :
              updateStatus === 'available' ? 'bg-success/10 text-success' :
              updateStatus === 'no-update' ? 'bg-info/10 text-info' :
              'bg-base-100'
            ]"
          >
            <AlertCircle v-if="updateStatus === 'error'" :size="12" />
            <CheckCircle v-else-if="updateStatus === 'no-update'" :size="12" />
            <RefreshCw v-else-if="updateStatus === 'checking'" :size="12" class="animate-spin" />
            <span>{{ updateMessage }}</span>
          </div>
        </div>

        <!-- 有新版本时显示下载按钮 -->
        <div v-if="updateStatus === 'available'" class="mt-3">
          <Button
            @click="openReleases"
            variant="primary"
            size="sm"
            class="w-full"
          >
            <Download :size="14" />
            前往下载最新版本
            <ExternalLink :size="12" />
          </Button>
        </div>
      </div>
    </div>

    <!-- 联系方式 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-3">
          <MessageCircle :size="16" />
          联系我们
        </h2>

        <!-- 直接显示公众号二维码 -->
        <div class="flex flex-col items-center">
          <div class="w-32 h-32 rounded-lg overflow-hidden border border-base-300 bg-base-100">
            <img
              :src="wechatQR"
              alt="微信公众号二维码"
              class="w-full h-full object-contain"
            />
          </div>
          <p class="text-sm font-medium mt-3">微信公众号</p>
          <p class="text-xs text-base-content/60 mt-1">linkron</p>
          <p class="text-xs text-base-content/40 mt-2">扫描关注公众号获取最新资讯</p>
        </div>
      </div>
    </div>

    <!-- 致谢 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-2">
          <Heart :size="16" />
          致谢
        </h2>
        <p class="text-xs text-base-content/60 leading-relaxed">
          感谢所有为 LINKRON 做出贡献的开发者和用户。特别感谢以下开源项目：
        </p>
        <div class="flex flex-wrap gap-2 mt-2">
          <span class="badge badge-xs badge-ghost">Tauri</span>
          <span class="badge badge-xs badge-ghost">Vue 3</span>
          <span class="badge badge-xs badge-ghost">DaisyUI</span>
          <span class="badge badge-xs badge-ghost">TipTap</span>
          <span class="badge badge-xs badge-ghost">SQLite</span>
        </div>
      </div>
    </div>
  </div>
</template>

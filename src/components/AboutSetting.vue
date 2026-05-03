<script setup>
import { ref, onMounted } from 'vue'
import { Info, MessageCircle, Heart, Download, RefreshCw, CheckCircle, AlertCircle, Settings } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'
import { useSettingStore } from '@/store/settingStore'
import wechatQR from '@/assets/weixin_gz.jpg'
import appLogo from '@/assets/128x128.png'

const settingStore = useSettingStore()

const version = ref('1.0.0')
const buildDate = '2025-02-16'

// 更新相关状态
const updateStatus = ref('idle') // idle, checking, available, downloading, installing, updated, error, no-update
const updateInfo = ref(null)
const updateProgress = ref(0)
const updateMessage = ref('')

// 自动更新设置
const autoUpdate = ref(false)

// 获取应用版本和设置
onMounted(async () => {
  try {
    version.value = await getVersion()
    // 加载自动更新设置
    autoUpdate.value = await settingStore.get('autoUpdate', true)
  } catch (e) {
    console.error('Failed to get version:', e)
  }
})

// 切换自动更新
async function toggleAutoUpdate() {
  autoUpdate.value = !autoUpdate.value
  await settingStore.set('autoUpdate', autoUpdate.value)
}

// 检查更新
async function checkForUpdate() {
  updateStatus.value = 'checking'
  updateMessage.value = '正在检查更新...'

  try {
    // Tauri 2.x API
    const update = await check()

    if (update) {
      updateInfo.value = update
      updateStatus.value = 'available'
      updateMessage.value = `发现新版本 ${update.version}`
    } else {
      updateStatus.value = 'no-update'
      updateMessage.value = '已是最新版本'
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

// 下载并安装更新
async function downloadAndInstallUpdate() {
  if (!updateInfo.value) return

  updateStatus.value = 'downloading'
  updateProgress.value = 0
  updateMessage.value = '正在下载更新...'

  try {
    await updateInfo.value.downloadAndInstall((progress) => {
      if (progress && typeof progress === 'object') {
        const { downloaded, contentLength } = progress
        if (downloaded !== undefined && contentLength !== undefined) {
          updateProgress.value = Math.round((downloaded / contentLength) * 100)
          updateMessage.value = `正在下载更新... ${updateProgress.value}%`
        }
      }
    })

    updateStatus.value = 'installing'
    updateMessage.value = '更新已安装，正在重启应用...'

    // 延迟一下让用户看到安装完成的消息
    await new Promise(resolve => setTimeout(resolve, 1500))

    await relaunch()
  } catch (error) {
    updateStatus.value = 'error'
    updateMessage.value = `更新失败: ${error?.message || String(error)}`
    console.error('Update install failed:', error)
  }
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
              <span class="text-xs text-base-content/40">版本 {{ version }}</span>
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
          <Button
            v-if="updateStatus === 'idle' || updateStatus === 'error' || updateStatus === 'no-update'"
            @click="checkForUpdate"
            variant="ghost"
            size="sm"
            :loading="updateStatus === 'checking'"
          >
            <RefreshCw :size="12" />
            检查更新
          </Button>
        </div>

        <!-- 自动更新开关 -->
        <div class="flex items-center justify-between p-3 rounded-lg bg-base-100">
          <div class="flex items-center gap-2">
            <RefreshCw :size="14" class="text-base-content/40" />
            <span class="text-sm">自动检查更新</span>
          </div>
          <input
            type="checkbox"
            class="toggle toggle-sm toggle-primary"
            :checked="autoUpdate"
            @change="toggleAutoUpdate"
          />
        </div>

        <!-- 更新状态消息 -->
        <div v-if="updateMessage" class="mt-3 mb-3">
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
            <Download v-else-if="updateStatus === 'downloading' || updateStatus === 'installing'" :size="12" :class="{ 'animate-pulse': updateStatus === 'downloading' }" />
            <RefreshCw v-else-if="updateStatus === 'checking'" :size="12" class="animate-spin" />
            <CheckCircle v-else-if="updateStatus === 'available'" :size="12" />
            <span>{{ updateMessage }}</span>
          </div>
        </div>

        <!-- 更新信息卡片 -->
        <div v-if="updateStatus === 'available' && updateInfo" class="bg-base-100 rounded-lg p-3 space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs text-base-content/60">新版本</span>
            <span class="text-sm font-medium">{{ updateInfo.version }}</span>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-xs text-base-content/60">发布日期</span>
            <span class="text-xs">{{ updateInfo.date }}</span>
          </div>
          <div v-if="updateInfo.body" class="mt-2 pt-2 border-t border-base-300">
            <p class="text-xs text-base-content/60 mb-1">更新内容</p>
            <p class="text-xs text-base-content/80 leading-relaxed">{{ updateInfo.body }}</p>
          </div>
          <div class="flex gap-2 mt-3">
            <Button
              @click="downloadAndInstallUpdate"
              variant="primary"
              size="sm"
              class="flex-1"
            >
              <Download :size="12" />
              立即更新
            </Button>
            <Button
              @click="updateStatus = 'idle'; updateMessage = ''; updateInfo = null"
              variant="ghost"
              size="sm"
            >
              稍后
            </Button>
          </div>
        </div>

        <!-- 下载进度 -->
        <div v-if="updateStatus === 'downloading'" class="bg-base-100 rounded-lg p-3">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs text-base-content/60">下载进度</span>
            <span class="text-xs font-medium">{{ updateProgress }}%</span>
          </div>
          <div class="w-full bg-base-300 rounded-full h-2">
            <div
              class="bg-primary h-2 rounded-full transition-all duration-300"
              :style="{ width: `${updateProgress}%` }"
            ></div>
          </div>
        </div>

        <!-- 安装中 -->
        <div v-if="updateStatus === 'installing'" class="bg-base-100 rounded-lg p-3">
          <div class="flex items-center gap-2">
            <RefreshCw :size="14" class="animate-spin text-primary" />
            <span class="text-xs text-base-content/60">正在安装更新，应用将自动重启...</span>
          </div>
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

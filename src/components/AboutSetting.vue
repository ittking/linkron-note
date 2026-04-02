<script setup>
import { ref, onMounted } from 'vue'
import { Info, Mail, MessageCircle, ExternalLink, Heart, Code, Zap, Download, RefreshCw, CheckCircle, AlertCircle } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'

const version = ref('1.0.0')
const buildDate = '2025-02-16'

// 更新相关状态
const updateStatus = ref('idle') // idle, checking, available, downloading, installing, updated, error, no-update
const updateInfo = ref(null)
const updateProgress = ref(0)
const updateMessage = ref('')

// 获取应用版本
onMounted(async () => {
  try {
    version.value = await getVersion()
  } catch (e) {
    console.error('Failed to get version:', e)
  }
})

// 检查更新
async function checkForUpdate() {
  updateStatus.value = 'checking'
  updateMessage.value = '正在检查更新...'

  try {
    const update = await check({
      headers: { Accept: 'application/json' }
    })

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
    updateMessage.value = `检查更新失败: ${error.message}`
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
    await updateInfo.value.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          updateMessage.value = '开始下载更新...'
          break
        case 'Progress':
          updateProgress.value = Math.round((event.data.downloaded / event.data.contentLength) * 100)
          updateMessage.value = `正在下载更新... ${updateProgress.value}%`
          break
        case 'Finished':
          updateMessage.value = '下载完成，正在安装...'
          break
      }
    })

    updateStatus.value = 'installing'
    updateMessage.value = '更新已安装，正在重启应用...'

    // 延迟一下让用户看到安装完成的消息
    await new Promise(resolve => setTimeout(resolve, 1500))

    await relaunch()
  } catch (error) {
    updateStatus.value = 'error'
    updateMessage.value = `更新失败: ${error.message}`
    console.error('Update install failed:', error)
  }
}

const contactInfo = {
  email: 'support@linkron.app',
  wechat: 'linkron_official',
  qqGroup: '123456789',
  github: 'https://github.com/ittking/linkron',
  website: 'https://linkron.app'
}

const features = [
  {
    icon: Zap,
    title: '高效笔记',
    description: '快速创建、管理和搜索您的笔记'
  },
  {
    icon: MessageCircle,
    title: 'AI 助手',
    description: '智能优化内容，提升笔记质量'
  },
  {
    icon: Code,
    title: '标签系统',
    description: '灵活的标签分类，快速定位笔记'
  }
]
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
          <div class="w-16 h-16 rounded-xl bg-gradient-to-br from-primary to-secondary flex items-center justify-center">
            <span class="text-2xl font-bold text-base-content">L</span>
          </div>
          <div>
            <h3 class="text-lg font-semibold">LINKRON</h3>
            <p class="text-sm text-base-content/60">智能笔记助手</p>
            <div class="flex items-center gap-2 mt-1">
              <span class="text-xs text-base-content/40">版本 {{ version }}</span>
              <span class="text-xs text-base-content/30">|</span>
              <span class="text-xs text-base-content/40">{{ buildDate }}</span>
            </div>
          </div>
        </div>

        <p class="text-xs text-base-content/60 leading-relaxed mb-4">
          LINKRON 是一款基于 Tauri + Vue 3 构建的跨平台桌面应用，旨在提供高效、智能的笔记管理体验。
        </p>

        <!-- 更新区域 -->
        <div class="border-t border-base-300 pt-4">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-medium">自动更新</span>
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

          <!-- 更新状态消息 -->
          <div v-if="updateMessage" class="mb-3">
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
    </div>

    <!-- 核心功能 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-3">
          <Zap :size="16" />
          核心功能
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <div
            v-for="feature in features"
            :key="feature.title"
            class="p-3 rounded-lg bg-base-100 border border-base-300"
          >
            <div class="flex items-center gap-2 mb-2">
              <component :is="feature.icon" :size="14" class="text-primary" />
              <span class="text-sm font-medium">{{ feature.title }}</span>
            </div>
            <p class="text-xs text-base-content/60">{{ feature.description }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 联系方式 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-3">
          <Mail :size="16" />
          联系我们
        </h2>
        <div class="space-y-2">
          <div class="flex items-center justify-between p-2 rounded-lg bg-base-100">
            <div class="flex items-center gap-2">
              <Mail :size="14" class="text-base-content/40" />
              <span class="text-sm">官方邮箱</span>
            </div>
            <a
              :href="`mailto:${contactInfo.email}`"
              class="text-xs text-primary hover:text-primary/80 transition-colors"
            >
              {{ contactInfo.email }}
            </a>
          </div>

          <div class="flex items-center justify-between p-2 rounded-lg bg-base-100">
            <div class="flex items-center gap-2">
              <MessageCircle :size="14" class="text-base-content/40" />
              <span class="text-sm">微信公众号</span>
            </div>
            <span class="text-xs text-base-content/60">{{ contactInfo.wechat }}</span>
          </div>

          <div class="flex items-center justify-between p-2 rounded-lg bg-base-100">
            <div class="flex items-center gap-2">
              <MessageCircle :size="14" class="text-base-content/40" />
              <span class="text-sm">QQ 群</span>
            </div>
            <span class="text-xs text-base-content/60">{{ contactInfo.qqGroup }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 相关链接 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-3">
          <ExternalLink :size="16" />
          相关链接
        </h2>
        <div class="space-y-2">
          <a
            :href="contactInfo.website"
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center justify-between p-2 rounded-lg bg-base-100 hover:bg-base-200 transition-colors group"
          >
            <span class="text-sm">官方网站</span>
            <ExternalLink :size="12" class="text-base-content/40 group-hover:text-primary transition-colors" />
          </a>

          <a
            :href="contactInfo.github"
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center justify-between p-2 rounded-lg bg-base-100 hover:bg-base-200 transition-colors group"
          >
            <span class="text-sm">GitHub 仓库</span>
            <ExternalLink :size="12" class="text-base-content/40 group-hover:text-primary transition-colors" />
          </a>
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

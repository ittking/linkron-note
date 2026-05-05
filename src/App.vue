<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useSettingStore } from '@/store/settingStore'
import { useAutoUpdater } from '@/composables/useAutoUpdater'
import { useReminder } from '@/composables/useReminder'
import { Download, ExternalLink, X } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'

const RELEASES_URL = 'https://github.com/ittking/linkron-note/releases'

const settingStore = useSettingStore()
const appVersion = ref('')

const { updateAvailable, latestVersion, startAutoCheck, stopAutoCheck } = useAutoUpdater(appVersion)
const { startReminderCheck, stopReminderCheck } = useReminder()

onMounted(async () => {
  const currentWindow = getCurrentWindow()
  const windowLabel = currentWindow.label

  // 加载并应用主题
  try {
    const theme = await settingStore.get('theme', 'light')
    document.documentElement.setAttribute('data-theme', theme)
  } catch (error) {
    console.error('Failed to load theme:', error)
  }

  // 获取版本号并启动自动更新检查
  try {
    appVersion.value = await getVersion()
    startAutoCheck()
  } catch (error) {
    console.error('Failed to get version:', error)
  }

  // 启动待办提醒检查
  startReminderCheck()
})

onUnmounted(() => {
  stopAutoCheck()
  stopReminderCheck()
})

function openReleases() {
  openUrl(RELEASES_URL)
}
</script>

<template>
  <div class="h-full">
    <router-view />
  </div>

  <!-- 更新提示弹窗 -->
  <Teleport to="body">
    <div
      v-if="updateAvailable"
      class="fixed inset-0 bg-base-content/20 backdrop-blur-sm flex items-center justify-center z-50"
    >
      <div class="bg-base-100 rounded-2xl shadow-xl w-85 p-6">
        <div class="flex items-start justify-between mb-4">
          <div>
            <h3 class="text-base font-semibold">发现新版本</h3>
            <p class="text-xs text-base-content/50 mt-1">最新版本已发布，建议您更新</p>
          </div>
          <button
            class="btn btn-ghost btn-xs btn-circle"
            @click="updateAvailable = false"
          >
            <X :size="14" />
          </button>
        </div>

        <div class="bg-base-200 rounded-xl p-4 mb-4">
          <div class="flex items-center justify-between">
            <span class="text-sm text-base-content/60">当前版本</span>
            <span class="text-sm font-medium">{{ appVersion }}</span>
          </div>
          <div class="flex items-center justify-between mt-2">
            <span class="text-sm text-base-content/60">最新版本</span>
            <span class="text-sm font-medium text-success">v{{ latestVersion }}</span>
          </div>
        </div>

        <div class="flex gap-3">
          <Button
            variant="ghost"
            size="sm"
            class="flex-1"
            @click="updateAvailable = false"
          >
            稍后提醒
          </Button>
          <Button
            variant="primary"
            size="sm"
            class="flex-1"
            @click="openReleases"
          >
            <Download :size="14" />
            前往下载
            <ExternalLink :size="12" />
          </Button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style>
html, body {
  height: 100%;
  margin: 0;
  padding: 0;
  overflow: hidden;
}

#app {
  height: 100%;
}
</style>

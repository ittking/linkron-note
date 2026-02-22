<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { User, LogOut, Crown, Calendar, Clock, Cloud, ChevronDown, ChevronUp } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from '../store/settingStore'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'
import dayjs from 'dayjs'

const settingStore = useSettingStore()

// 用户信息
const userInfo = ref({
  nickname: '',
  avatar: '',
  isVip: false,
  loginTime: ''
})

// 同步配置
const syncConfig = ref({
  platform: 'gitee',
  token: '',
  repoUrl: '',
  branch: 'main',
  autoSyncDelay: 3000 // 自动同步延迟（毫秒），默认3秒
})

// 平台选项
const platforms = [
  { value: 'gitee', label: 'Gitee', url: 'https://gitee.com/用户名/仓库名.git' },
  { value: 'github', label: 'GitHub', url: 'https://github.com/用户名/仓库名.git' },
  { value: 'custom', label: '自定义', url: '' }
]

// 状态
const isConfigured = ref(false)
const isTesting = ref(false)
const isSyncing = ref(false)
const testResult = ref(null)
const showSyncConfig = ref(false)
const lastSyncTime = ref(null)
const syncStatus = ref(null)

// 自动同步定时器
let autoSyncTimer = null
let autoSyncTimeout = null

// 工作目录
const workDirectory = ref('')

// 预设的自动同步延迟选项
const autoSyncDelayOptions = [
  { label: '关闭', value: 0 },
  { label: '2秒', value: 2000 },
  { label: '3秒', value: 3000 },
  { label: '5秒', value: 5000 },
  { label: '10秒', value: 10000 }
]

// 初始化
onMounted(async () => {
  await loadUserInfo()
  await loadWorkDirectory()
  await loadSyncConfig()
  await loadLastSyncTime()
})

// 组件卸载时清除定时器
onUnmounted(() => {
  clearAutoSync()
})

// 格式化登录时间
const formattedLoginTime = computed(() => {
  if (!userInfo.value.loginTime) return ''
  const date = new Date(userInfo.value.loginTime)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
})

// 格式化上次同步时间
const formattedLastSyncTime = computed(() => {
  if (!lastSyncTime.value) return '从未同步'
  return dayjs(lastSyncTime.value).format('YYYY-MM-DD HH:mm:ss')
})

// 加载用户信息
async function loadUserInfo() {
  try {
    const saved = await settingStore.get('userInfo', {
      nickname: '用户',
      avatar: '',
      isVip: false,
      loginTime: new Date().toISOString()
    })
    userInfo.value = saved || {
      nickname: '用户',
      avatar: '',
      isVip: false,
      loginTime: new Date().toISOString()
    }
  } catch (error) {
    console.error('Failed to load user info:', error)
    userInfo.value = {
      nickname: '用户',
      avatar: '',
      isVip: false,
      loginTime: new Date().toISOString()
    }
  }
}

// 加载工作目录
async function loadWorkDirectory() {
  try {
    workDirectory.value = await settingStore.get('workDirectory', '')
  } catch (error) {
    console.error('Failed to load work directory:', error)
  }
}

// 加载同步配置
async function loadSyncConfig() {
  try {
    const config = await invoke('get_sync_config', { workDirectory: workDirectory.value })
    if (config) {
      syncConfig.value = {
        ...syncConfig.value,
        ...config,
        autoSyncDelay: config.autoSyncDelay || 3000
      }
      isConfigured.value = true

      // 如果配置了自动同步，启动自动同步
      if (syncConfig.value.autoSyncDelay > 0) {
        startAutoSync()
      }
    }
  } catch (error) {
    console.error('Failed to load sync config:', error)
  }
}

// 加载上次同步时间
async function loadLastSyncTime() {
  try {
    const savedTime = await settingStore.get('lastSyncTime', null)
    if (savedTime) {
      lastSyncTime.value = savedTime
    }
  } catch (error) {
    console.error('Failed to load last sync time:', error)
  }
}

// 退出登录
async function handleLogout() {
  if (confirm('确定要退出登录吗？')) {
    await settingStore.set('isAuthenticated', false)
    window.location.reload()
  }
}

// 保存同步配置
async function saveSyncConfig() {
  if (!syncConfig.value.token || !syncConfig.value.repoUrl) {
    syncStatus.value = {
      type: 'error',
      message: '请填写完整的配置信息'
    }
    setTimeout(() => { syncStatus.value = null }, 3000)
    return
  }

  try {
    await invoke('save_sync_config', {
      config: syncConfig.value,
      workDirectory: workDirectory.value
    })
    isConfigured.value = true

    // 更新自动同步设置
    clearAutoSync()
    if (syncConfig.value.autoSyncDelay > 0) {
      startAutoSync()
    }

    syncStatus.value = {
      type: 'success',
      message: '配置已保存'
    }
    setTimeout(() => { syncStatus.value = null }, 2000)
  } catch (error) {
    console.error('Failed to save config:', error)
    syncStatus.value = {
      type: 'error',
      message: '保存失败: ' + error
    }
    setTimeout(() => { syncStatus.value = null }, 3000)
  }
}

// 检测连接
async function testConnection() {
  if (!syncConfig.value.token || !syncConfig.value.repoUrl) {
    testResult.value = {
      success: false,
      message: '请先填写 Token 和仓库地址'
    }
    return
  }

  isTesting.value = true
  testResult.value = null

  try {
    const result = await invoke('test_git_connection', {
      config: syncConfig.value,
      workDirectory: workDirectory.value
    })
    testResult.value = result
  } catch (error) {
    console.error('Failed to test connection:', error)
    testResult.value = {
      success: false,
      message: '测试失败: ' + error
    }
  } finally {
    isTesting.value = false
  }
}

// 立即同步（推送）
async function syncNow() {
  if (!isConfigured.value) {
    syncStatus.value = {
      type: 'error',
      message: '请先配置并保存同步信息'
    }
    setTimeout(() => { syncStatus.value = null }, 3000)
    return
  }

  isSyncing.value = true

  try {
    const result = await invoke('sync_to_remote', {
      config: syncConfig.value,
      workDirectory: workDirectory.value
    })

    // 更新上次同步时间
    lastSyncTime.value = new Date().toISOString()
    await settingStore.set('lastSyncTime', lastSyncTime.value)

    // 重启自动同步计时器
    if (syncConfig.value.autoSyncDelay > 0) {
      restartAutoSync()
    }

    syncStatus.value = {
      type: 'success',
      message: result.message || '同步成功'
    }
    setTimeout(() => { syncStatus.value = null }, 2000)
  } catch (error) {
    console.error('Failed to sync:', error)
    syncStatus.value = {
      type: 'error',
      message: '同步失败: ' + error
    }
    setTimeout(() => { syncStatus.value = null }, 3000)
  } finally {
    isSyncing.value = false
  }
}

// 覆盖本地（拉取）
async function overwriteLocal() {
  if (!isConfigured.value) {
    syncStatus.value = {
      type: 'error',
      message: '请先配置并保存同步信息'
    }
    setTimeout(() => { syncStatus.value = null }, 3000)
    return
  }

  if (!confirm('确定要从远程拉取数据？这将会覆盖本地所有更改，此操作不可撤销！')) {
    return
  }

  isSyncing.value = true

  try {
    const result = await invoke('sync_from_remote', {
      config: syncConfig.value,
      workDirectory: workDirectory.value
    })

    // 更新上次同步时间
    lastSyncTime.value = new Date().toISOString()
    await settingStore.set('lastSyncTime', lastSyncTime.value)

    // 重启自动同步计时器
    if (syncConfig.value.autoSyncDelay > 0) {
      restartAutoSync()
    }

    syncStatus.value = {
      type: 'success',
      message: result.message || '已覆盖本地数据'
    }
    setTimeout(() => { syncStatus.value = null }, 2000)
  } catch (error) {
    console.error('Failed to sync:', error)
    syncStatus.value = {
      type: 'error',
      message: '同步失败: ' + error
    }
    setTimeout(() => { syncStatus.value = null }, 3000)
  } finally {
    isSyncing.value = false
  }
}

// 启动自动同步
function startAutoSync() {
  if (autoSyncTimer) return // 避免重复启动

  const delay = syncConfig.value.autoSyncDelay
  if (delay <= 0) return

  // 设置延迟后自动同步
  autoSyncTimeout = setTimeout(() => {
    performAutoSync()
  }, delay)
}

// 清除自动同步
function clearAutoSync() {
  if (autoSyncTimeout) {
    clearTimeout(autoSyncTimeout)
    autoSyncTimeout = null
  }
  if (autoSyncTimer) {
    clearTimeout(autoSyncTimer)
    autoSyncTimer = null
  }
}

// 重启自动同步
function restartAutoSync() {
  clearAutoSync()
  startAutoSync()
}

// 执行自动同步（静默同步，不显示提示）
async function performAutoSync() {
  if (!isConfigured.value) return

  try {
    await invoke('sync_to_remote', {
      config: syncConfig.value,
      workDirectory: workDirectory.value
    })

    // 更新上次同步时间
    lastSyncTime.value = new Date().toISOString()
    await settingStore.set('lastSyncTime', lastSyncTime.value)

    // 继续下一次自动同步
    startAutoSync()
  } catch (error) {
    console.error('Auto sync failed:', error)
    // 自动同步失败后，继续尝试下一次
    startAutoSync()
  }
}

// 平台变化时更新示例URL
function onPlatformChange() {
  const platform = platforms.find(p => p.value === syncConfig.value.platform)
  if (platform && platform.url) {
    syncConfig.value.repoUrl = platform.url
  }
}
</script>

<template>
  <div class="space-y-4">
    <!-- 账户信息卡片 -->
    <div class="card bg-base-200 shadow-sm rounded-2xl overflow-hidden relative">
      <!-- 退出登录按钮 -->
      <button
        @click="handleLogout"
        class="absolute top-3 right-3 w-8 h-8 rounded-lg flex items-center justify-center text-base-content/40 hover:text-error hover:bg-error/5 transition-colors"
        title="退出登录"
      >
        <LogOut :size="16" />
      </button>

      <div class="card-body p-4 space-y-4">
        <!-- 头像和基本信息 -->
        <div class="flex items-center gap-4">
          <!-- 头像 -->
          <div class="relative flex-shrink-0">
            <div
              class="w-14 h-14 rounded-xl bg-gradient-to-br from-primary/10 to-primary/5 flex items-center justify-center overflow-hidden"
              :class="userInfo.isVip ? 'ring-2 ring-amber-400/50 ring-offset-2 ring-offset-base-200' : ''"
            >
              <img v-if="userInfo.avatar" :src="userInfo.avatar" alt="头像" class="w-full h-full object-cover" />
              <User v-else :size="24" class="text-primary/50" />
            </div>
            <!-- VIP 徽章 -->
            <div
              v-if="userInfo.isVip"
              class="absolute -top-1 -right-1 w-5 h-5 bg-gradient-to-br from-amber-400 to-yellow-500 rounded-full flex items-center justify-center"
            >
              <Crown :size="9" class="text-white" />
            </div>
          </div>

          <!-- 用户信息 -->
          <div class="flex-1 min-w-0">
            <div class="space-y-0.5">
              <!-- 昵称 -->
              <h3 class="text-base font-semibold text-base-content truncate">{{ userInfo.nickname || '用户' }}</h3>

              <!-- 会员状态 -->
              <div class="flex items-center gap-2">
                <span
                  class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium"
                  :class="userInfo.isVip
                    ? 'bg-gradient-to-r from-amber-500/10 to-yellow-500/10 text-amber-600 dark:text-amber-400 border border-amber-500/20'
                    : 'bg-base-100 text-base-content/60'"
                >
                  <Crown v-if="userInfo.isVip" :size="10" />
                  {{ userInfo.isVip ? 'VIP 会员' : '普通用户' }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- 信息项 -->
        <div class="flex items-center gap-3 p-2.5 rounded-lg bg-base-100/50">
          <div class="w-7 h-7 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
            <Calendar :size="14" class="text-primary/70" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-xs text-base-content/50 mb-0.5">登录时间</p>
            <p class="text-sm text-base-content/80 truncate">{{ formattedLoginTime || '未记录' }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 云同步卡片 -->
    <div class="card bg-base-200 shadow-sm rounded-2xl overflow-hidden">
      <div class="card-body p-4 space-y-4">
        <!-- 标题和上次同步时间 -->
        <div class="flex items-center justify-between">
          <h2 class="card-title text-sm font-medium flex items-center gap-2">
            <Cloud :size="16" />
            云同步
          </h2>
          <div v-if="lastSyncTime" class="flex items-center gap-1.5 text-xs text-base-content/50">
            <Clock :size="12" />
            <span>{{ formattedLastSyncTime }}</span>
          </div>
        </div>

        <!-- 同步操作按钮 -->
        <div class="flex gap-2">
          <Button
            variant="primary"
            size="sm"
            @click="syncNow"
            :disabled="isSyncing || !isConfigured"
            class="flex-1"
          >
            {{ isSyncing ? '同步中...' : '立即同步' }}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            @click="overwriteLocal"
            :disabled="isSyncing || !isConfigured"
            class="flex-1"
          >
            覆盖本地
          </Button>
          <Button
            variant="ghost"
            size="sm"
            @click="showSyncConfig = !showSyncConfig"
            class="gap-1"
          >
            {{ showSyncConfig ? '收起' : '配置' }}
            <ChevronDown v-if="!showSyncConfig" :size="14" />
            <ChevronUp v-else :size="14" />
          </Button>
        </div>

        <!-- 状态提示 -->
        <div
          v-if="syncStatus"
          class="flex items-center gap-2 p-2.5 rounded-lg text-xs"
          :class="syncStatus.type === 'success' ? 'bg-success/10 text-success' : 'bg-error/10 text-error'"
        >
          {{ syncStatus.message }}
        </div>

        <!-- 配置表单 -->
        <div v-if="showSyncConfig" class="space-y-3 pt-3 border-t border-base-300">
          <!-- 平台选择 -->
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">同步平台</span>
            </label>
            <div class="flex gap-2">
              <button
                v-for="platform in platforms"
                :key="platform.value"
                @click="syncConfig.platform = platform.value; onPlatformChange()"
                class="flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-colors"
                :class="syncConfig.platform === platform.value
                  ? 'bg-primary text-primary-content'
                  : 'bg-base-100 text-base-content/60 hover:bg-base-100/80'"
              >
                {{ platform.label }}
              </button>
            </div>
          </div>

          <!-- Token 输入 -->
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">访问令牌 (Token)</span>
            </label>
            <Input
              type="password"
              v-model="syncConfig.token"
              placeholder="输入您的访问令牌"
              size="sm"
            />
          </div>

          <!-- 仓库地址 -->
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">仓库地址</span>
            </label>
            <Input
              type="text"
              v-model="syncConfig.repoUrl"
              placeholder="https://github.com/username/repo.git"
              size="sm"
            />
          </div>

          <!-- 分支名称 -->
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">分支名称</span>
            </label>
            <Input
              type="text"
              v-model="syncConfig.branch"
              placeholder="main"
              size="sm"
            />
          </div>

          <!-- 自动同步延迟 -->
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">自动同步</span>
            </label>
            <div class="flex gap-2">
              <button
                v-for="option in autoSyncDelayOptions"
                :key="option.value"
                @click="syncConfig.autoSyncDelay = option.value"
                class="flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-colors"
                :class="syncConfig.autoSyncDelay === option.value
                  ? 'bg-primary text-primary-content'
                  : 'bg-base-100 text-base-content/60 hover:bg-base-100/80'"
              >
                {{ option.label }}
              </button>
            </div>
            <label class="label">
              <span class="label-text-alt text-[11px] text-base-content/40">
                更新后自动同步到云端
              </span>
            </label>
          </div>

          <!-- 操作按钮 -->
          <div class="flex gap-2">
            <Button
              variant="primary"
              size="sm"
              @click="testConnection"
              :disabled="isTesting"
              class="flex-1"
            >
              {{ isTesting ? '检测中...' : '检测连接' }}
            </Button>
            <Button
              variant="primary"
              size="sm"
              @click="saveSyncConfig"
              class="flex-1"
            >
              保存配置
            </Button>
          </div>

          <!-- 测试结果 -->
          <div
            v-if="testResult"
            class="flex items-center gap-2 p-3 rounded-lg text-xs"
            :class="testResult.success ? 'bg-success/10 text-success' : 'bg-error/10 text-error'"
          >
            <span>{{ testResult.message }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

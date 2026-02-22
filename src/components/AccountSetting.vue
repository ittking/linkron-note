<script setup>
import { ref, computed, onMounted } from 'vue'
import { User, LogOut, Crown, Calendar, Cloud, CloudOff, Settings, Check } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from '../store/settingStore'
import Button from './ui/Button.vue'
import Input from './ui/Input.vue'

const settingStore = useSettingStore()

// 用户信息
const userInfo = ref({
  nickname: '',
  avatar: '',
  isVip: false,
  loginTime: ''
})

// 初始化
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

// 退出登录
async function handleLogout() {
  if (confirm('确定要退出登录吗？')) {
    await settingStore.set('isAuthenticated', false)
    window.location.reload()
  }
}

// 初始化加载
loadUserInfo()

// ===== 云同步配置 =====
const syncConfig = ref({
  repo_url: '',  // 仓库地址
  token: '',     // 访问令牌
  branch: 'main' // 分支（Gitee/GitHub 新仓库默认都是 main）
})

const isConfigured = ref(false)
const isSyncing = ref(false)
const showConfig = ref(false)
const lastSyncTime = ref(null)
const connectionStatus = ref('')
const connectionSuccess = ref(false)

// 格式化同步时间
const formattedLastSyncTime = computed(() => {
  if (!lastSyncTime.value) return ''
  const date = new Date(lastSyncTime.value * 1000)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
})

// 加载已保存的同步配置
async function loadSyncConfig() {
  try {
    const config = await invoke('get_sync_config')
    if (config) {
      syncConfig.value = config
      isConfigured.value = true
      loadSyncTime()
    }
  } catch (error) {
    console.error('Failed to load sync config:', error)
  }
}

// 加载同步时间（从设置中读取）
async function loadSyncTime() {
  try {
    // 从设置中读取上次同步时间
    const syncTime = await settingStore.get('lastSyncTime', null)
    if (syncTime) {
      lastSyncTime.value = syncTime
    }
  } catch (error) {
    console.error('Failed to load sync time:', error)
  }
}

// 保存同步配置
async function saveConfig() {
  try {
    await invoke('save_sync_config', { config: syncConfig.value })
    isConfigured.value = true
    showConfig.value = false
    alert('配置保存成功')
  } catch (error) {
    alert('保存配置失败: ' + error)
  }
}

// 检测连接
async function testConnection() {
  connectionStatus.value = '检测中...'
  connectionSuccess.value = false

  try {
    const result = await invoke('validate_sync_config', { config: syncConfig.value })
    if (result.success) {
      connectionStatus.value = '连接成功'
      connectionSuccess.value = true
    } else {
      connectionStatus.value = result.message
      connectionSuccess.value = false
    }
  } catch (error) {
    connectionStatus.value = '检测失败: ' + error
    connectionSuccess.value = false
  }
}

// 推送到云端（先强制拉取，再推送）
async function syncToRemote() {
  if (!isConfigured.value) {
    alert('请先配置云同步')
    return
  }

  isSyncing.value = true
  try {
    const result = await invoke('sync_to_remote', {
      config: syncConfig.value,
      workDirectory: await settingStore.get('workDirectory')
    })

    if (result.success) {
      // 更新同步时间
      lastSyncTime.value = Math.floor(Date.now() / 1000)
      await settingStore.set('lastSyncTime', lastSyncTime.value)
      // 刷新页面以重新加载数据
      setTimeout(() => window.location.reload(), 500)
    } else {
      alert('同步失败: ' + result.message)
    }
  } catch (error) {
    alert('同步失败: ' + error)
  } finally {
    isSyncing.value = false
  }
}

// 初始化加载
onMounted(() => {
  loadSyncConfig()
})
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
        <!-- 标题和状态 -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <h2 class="card-title text-sm font-medium flex items-center gap-2">
              <Cloud :size="16" />
              云同步
            </h2>
            <div v-if="isConfigured" class="flex items-center gap-1">
              <Check :size="12" class="text-success" />
              <span class="text-xs text-success">已配置</span>
            </div>
            <div v-else class="flex items-center gap-1">
              <CloudOff :size="12" class="text-base-content/40" />
              <span class="text-xs text-base-content/40">未配置</span>
            </div>
          </div>
          <div v-if="lastSyncTime" class="text-xs text-base-content/50">
            上次同步: {{ formattedLastSyncTime }}
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="flex gap-2">
          <Button
            @click="syncToRemote"
            :disabled="isSyncing || !isConfigured"
            :loading="isSyncing"
            variant="primary"
            size="sm"
          >
            <Cloud :size="14" />
            立即同步
          </Button>
          <Button
            @click="showConfig = !showConfig"
            variant="ghost"
            size="sm"
          >
            <Settings :size="14" />
            {{ showConfig ? '收起配置' : '配置' }}
          </Button>
        </div>

        <!-- 配置表单 -->
        <div v-if="showConfig" class="space-y-3 pt-3 border-t border-base-300">
          <!-- 仓库地址 -->
          <div>
            <label class="text-xs text-base-content/60 mb-1.5 block">仓库地址</label>
            <Input
              v-model="syncConfig.repo_url"
              placeholder="例如：https://gitee.com/username/repo 或 username/repo"
              size="sm"
            />
            <p class="text-xs text-base-content/40 mt-1">
              支持 GitHub/Gitee 完整 URL 或简短格式（用户名/仓库名）
            </p>
          </div>

          <!-- Token -->
          <div>
            <label class="text-xs text-base-content/60 mb-1.5 block">访问令牌 (Token)</label>
            <Input
              v-model="syncConfig.token"
              type="password"
              placeholder="输入 Personal Access Token"
              size="sm"
            />
            <p class="text-xs text-base-content/40 mt-1">
              Token 需要仓库读写权限
            </p>
          </div>

          <!-- 分支 -->
          <div>
            <label class="text-xs text-base-content/60 mb-1.5 block">分支</label>
            <Input
              v-model="syncConfig.branch"
              placeholder="main"
              size="sm"
            />
            <p class="text-xs text-base-content/40 mt-1">
              新仓库默认为 main，老仓库可能是 master
            </p>
          </div>

          <!-- 连接测试状态 -->
          <div v-if="connectionStatus" :class="[
            'text-xs px-3 py-2 rounded-lg',
            connectionSuccess ? 'bg-success/10 text-success' : 'bg-error/10 text-error'
          ]">
            {{ connectionStatus }}
          </div>

          <!-- 操作按钮 -->
          <div class="flex gap-2 pt-2">
            <Button
              @click="testConnection"
              variant="secondary"
              size="sm"
              class="flex-1"
            >
              检测连接
            </Button>
            <Button
              @click="saveConfig"
              variant="primary"
              size="sm"
              class="flex-1"
              :disabled="!syncConfig.repo_url || !syncConfig.token"
            >
              保存配置
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

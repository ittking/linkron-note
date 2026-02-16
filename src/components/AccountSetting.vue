<script setup>
import { ref, onMounted, computed } from 'vue'
import { User, LogOut, Crown, Calendar } from 'lucide-vue-next'
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

// 同步配置
const syncConfig = ref({
  platform: 'gitee',
  token: '',
  repoUrl: '',
  branch: 'main'
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
const testResult = ref(null)
const isSyncing = ref(false)
const syncDirection = ref('push')
const showSyncConfig = ref(false)

// 工作目录
const workDirectory = ref('')

// 初始化
onMounted(async () => {
  await loadUserInfo()
  await loadWorkDirectory()
  await loadSyncConfig()
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
      syncConfig.value = config
      isConfigured.value = true
    }
  } catch (error) {
    console.error('Failed to load sync config:', error)
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
    alert('请填写完整的配置信息')
    return
  }

  try {
    await invoke('save_sync_config', {
      config: syncConfig.value,
      workDirectory: workDirectory.value
    })
    isConfigured.value = true
    alert('配置已保存')
  } catch (error) {
    console.error('Failed to save config:', error)
    alert('保存失败: ' + error)
  }
}

// 检测连接
async function testConnection() {
  if (!syncConfig.value.token || !syncConfig.value.repoUrl) {
    alert('请先填写 Token 和仓库地址')
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

// 同步到远程
async function syncToRemote() {
  if (!isConfigured.value) {
    alert('请先配置并保存同步信息')
    return
  }

  isSyncing.value = true
  syncDirection.value = 'push'

  try {
    const result = await invoke('sync_to_remote', {
      config: syncConfig.value,
      workDirectory: workDirectory.value
    })
    alert(result.message || '同步成功')
  } catch (error) {
    console.error('Failed to sync:', error)
    alert('同步失败: ' + error)
  } finally {
    isSyncing.value = false
  }
}

// 从远程同步
async function syncFromRemote() {
  if (!isConfigured.value) {
    alert('请先配置并保存同步信息')
    return
  }

  if (!confirm('确定要从远程拉取数据？这可能会覆盖本地更改。')) {
    return
  }

  isSyncing.value = true
  syncDirection.value = 'pull'

  try {
    const result = await invoke('sync_from_remote', {
      config: syncConfig.value,
      workDirectory: workDirectory.value
    })
    alert(result.message || '拉取成功')
  } catch (error) {
    console.error('Failed to sync:', error)
    alert('同步失败: ' + error)
  } finally {
    isSyncing.value = false
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

    <!-- 同步配置卡片 -->
    <div class="card bg-base-200 shadow-sm rounded-2xl overflow-hidden">
      <div class="card-body p-4 space-y-4">
        <h2 class="card-title text-sm font-medium flex items-center gap-2">
          <Calendar :size="16" />
          Git 同步
        </h2>

        <!-- 同步操作按钮 -->
        <div class="flex gap-2">
          <Button
            variant="primary"
            size="sm"
            @click="syncToRemote"
            :disabled="isSyncing || !isConfigured"
            class="flex-1"
          >
            推送
          </Button>
          <Button
            variant="primary"
            size="sm"
            @click="syncFromRemote"
            :disabled="isSyncing || !isConfigured"
            class="flex-1"
          >
            拉取
          </Button>
          <Button
            variant="ghost"
            size="sm"
            @click="showSyncConfig = !showSyncConfig"
            class="flex-1"
          >
            {{ showSyncConfig ? '收起' : '配置' }}
          </Button>
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
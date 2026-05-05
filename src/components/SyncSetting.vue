<script setup>
import { ref, onMounted } from 'vue'
import { Cloud, CloudUpload, CloudDownload, RefreshCw, Check, X, Loader2, Clock } from 'lucide-vue-next'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'
import Toggle from './ui/Toggle.vue'
import { useSync } from '../composables/useSync'
import { useToast } from '../composables/useToast'
import { useSettingStore } from '../store/settingStore'

const { showToast } = useToast()
const {
  isSyncing,
  lastSyncTime,
  formattedLastSyncTime,
  loadSyncTime,
  loadConfig,
  saveConfig,
  testConnection,
  syncToCloud,
  syncFromCloud
} = useSync()

// 表单数据
const formData = ref({
  repo_url: '',
  token: '',
  branch: 'main'
})

// 自动同步延时选项
const autoSyncDelayOptions = [
  { value: 0, label: '关闭' },
  { value: 5000, label: '5秒' },
  { value: 10000, label: '10秒' },
  { value: 30000, label: '30秒' },
  { value: 60000, label: '1分钟' }
]

const autoSyncDelay = ref(5000)
const isTesting = ref(false)
const testResult = ref(null)
const isSaving = ref(false)

// 初始化
onMounted(async () => {
  await loadSyncTime()
  const config = await loadConfig()
  formData.value = config

  // 加载自动同步延时配置
  try {
    const settingStore = useSettingStore()
    const value = await settingStore.get('autoSyncDelay', 5000)
    autoSyncDelay.value = value ?? 5000
  } catch (error) {
    console.error('Failed to load auto sync delay:', error)
  }
})

// 监听自动同步延时变化
async function handleAutoSyncDelayChange(value) {
  autoSyncDelay.value = Number(value)
  try {
    const settingStore = useSettingStore()
    await settingStore.set('autoSyncDelay', autoSyncDelay.value)
    showToast('自动同步设置已更新', 'success')
  } catch (error) {
    console.error('Failed to save auto sync delay:', error)
  }
}

// 测试连接
async function handleTestConnection() {
  if (!formData.value.repo_url || !formData.value.token) {
    showToast('请先填写仓库地址和Token', 'error')
    return
  }

  isTesting.value = true
  testResult.value = null

  try {
    const result = await testConnection(formData.value)
    testResult.value = result

    if (result.success) {
      showToast('连接测试成功', 'success')
    } else {
      showToast('连接测试失败: ' + result.message, 'error')
    }
  } catch (error) {
    testResult.value = {
      success: false,
      message: error.message || '连接测试失败'
    }
    showToast('连接测试失败: ' + error.message, 'error')
  } finally {
    isTesting.value = false
  }
}

// 保存配置
async function handleSaveConfig() {
  if (!formData.value.repo_url || !formData.value.token) {
    showToast('请填写完整的配置信息', 'error')
    return
  }

  isSaving.value = true

  try {
    const result = await saveConfig(formData.value)
    if (result.success) {
      showToast('配置保存成功', 'success')
    } else {
      showToast('配置保存失败: ' + result.error, 'error')
    }
  } catch (error) {
    showToast('配置保存失败: ' + error.message, 'error')
  } finally {
    isSaving.value = false
  }
}

// 同步到云端
async function handleSyncToCloud() {
  const result = await syncToCloud()
  if (result.success) {
    showToast('同步成功', 'success')
  } else {
    let msg = result.message
    // 附带失败文件名
    if (result.details?.failed_files?.length) {
      msg += '\n失败文件: ' + result.details.failed_files.join(', ')
    }
    showToast(msg, 'error')
  }
}

// 从云端同步
async function handleSyncFromCloud() {
  const result = await syncFromCloud()
  if (result.success) {
    showToast('同步成功，请刷新页面查看最新数据', 'success')
    // 刷新页面以加载最新数据
    setTimeout(() => {
      window.location.reload()
    }, 1000)
  } else {
    showToast('同步失败: ' + result.message, 'error')
  }
}

// 获取Gitee Token帮助链接
function getGiteeTokenHelpUrl() {
  return 'https://gitee.com/profile/personal_access_tokens'
}
</script>

<template>
  <div class="space-y-4">
    <!-- Gitee 同步配置 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between mb-4">
          <h2 class="card-title text-sm font-medium flex items-center gap-2">
            <Cloud :size="16" />
            Gitee 云同步
          </h2>
          <a
            :href="getGiteeTokenHelpUrl()"
            target="_blank"
            rel="noopener noreferrer"
            class="text-xs text-primary hover:underline"
          >
            如何获取 Token?
          </a>
        </div>

        <div class="space-y-3">
          <!-- 仓库地址 -->
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">仓库地址</span>
            </label>
            <Input
              v-model="formData.repo_url"
              placeholder="用户名/仓库名 或完整仓库 URL"
              size="sm"
            />
            <label class="label">
              <span class="label-text-alt text-[11px] text-base-content/40">
                支持格式：user/repo 或 https://gitee.com/user/repo.git
              </span>
            </label>
          </div>

          <!-- 访问令牌 -->
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">访问令牌 (Token)</span>
            </label>
            <Input
              v-model="formData.token"
              type="password"
              placeholder="输入 Gitee 访问令牌"
              size="sm"
            />
            <label class="label">
              <span class="label-text-alt text-[11px] text-base-content/40">
                需要仓库的读写权限
              </span>
            </label>
          </div>

          <!-- 分支 -->
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">分支</span>
            </label>
            <Input
              v-model="formData.branch"
              placeholder="默认为 main"
              size="sm"
            />
          </div>

          <!-- 测试连接结果 -->
          <div v-if="testResult" :class="[
            'p-3 rounded-lg text-sm',
            testResult.success ? 'bg-success/10 text-success' : 'bg-error/10 text-error'
          ]">
            <div class="flex items-center gap-2">
              <component :is="testResult.success ? Check : X" :size="16" />
              <span>{{ testResult.message }}</span>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="flex gap-2 pt-2">
            <Button
              variant="ghost"
              size="sm"
              class="flex-1"
              :disabled="isTesting || !formData.repo_url || !formData.token"
              @click="handleTestConnection"
            >
              <Loader2 v-if="isTesting" :size="14" class="animate-spin mr-1" />
              <RefreshCw v-else :size="14" class="mr-1" />
              测试连接
            </Button>
            <Button
              variant="primary"
              size="sm"
              class="flex-1"
              :disabled="isSaving || !formData.repo_url || !formData.token"
              @click="handleSaveConfig"
            >
              <Loader2 v-if="isSaving" :size="14" class="animate-spin mr-1" />
              <Check v-else :size="14" class="mr-1" />
              保存配置
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- 同步操作 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-4">同步操作</h2>

        <!-- 上次同步时间 -->
        <div v-if="lastSyncTime" class="flex items-center gap-2 mb-4 text-xs text-base-content/60">
          <Clock :size="14" />
          <span>上次同步: {{ formattedLastSyncTime }}</span>
        </div>

        <!-- 同步按钮 -->
        <div class="space-y-3">
          <Button
            variant="primary"
            size="sm"
            block
            :disabled="isSyncing || !formData.repo_url || !formData.token"
            @click="handleSyncToCloud"
          >
            <Loader2 v-if="isSyncing" :size="16" class="animate-spin mr-2" />
            <CloudUpload v-else :size="16" class="mr-2" />
            {{ isSyncing ? '同步中...' : '同步到云端' }}
          </Button>

          <Button
            variant="ghost"
            size="sm"
            block
            :disabled="isSyncing || !formData.repo_url || !formData.token"
            @click="handleSyncFromCloud"
          >
            <Loader2 v-if="isSyncing" :size="16" class="animate-spin mr-2" />
            <CloudDownload v-else :size="16" class="mr-2" />
            {{ isSyncing ? '同步中...' : '从云端同步' }}
          </Button>
        </div>

        <div class="mt-3 p-3 bg-base-300 rounded-lg">
          <p class="text-xs text-base-content/60 leading-relaxed">
            <strong class="text-base-content/80">同步说明：</strong><br>
            • <strong>同步到云端</strong>：通过 Gitee API 上传本地文件<br>
            • <strong>从云端同步</strong>：通过 Gitee API 下载文件到本地<br>
            • <strong>无需安装 Git</strong>：直接使用 API 操作文件<br>
            • <strong>文件限制</strong>：单个文件最大 10MB<br>
            • <strong>数据库文件</strong>：note.db 如果超过 10MB 将被跳过<br>
            • 首次同步请确保云端仓库为空，或先备份本地数据
          </p>
        </div>
      </div>
    </div>

    <!-- 自动同步设置 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium mb-4">自动同步</h2>

        <div class="space-y-3">
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">自动同步延时</span>
            </label>
            <div class="flex gap-2 flex-wrap">
              <Button
                v-for="option in autoSyncDelayOptions"
                :key="option.value"
                :variant="autoSyncDelay === option.value ? 'primary' : 'ghost'"
                size="sm"
                @click="handleAutoSyncDelayChange(option.value)"
              >
                {{ option.label }}
              </Button>
            </div>
            <label class="label">
              <span class="label-text-alt text-[11px] text-base-content/40">
                数据变更后延迟指定时间自动同步到云端
              </span>
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

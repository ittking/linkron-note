<script setup>
import { ref, onMounted, computed } from 'vue'
import { User, LogOut, Crown, Calendar } from 'lucide-vue-next'
import { useSettingStore } from '../store/settingStore'
import Button from './ui/Button.vue'

const settingStore = useSettingStore()

// 用户信息
const userInfo = ref({
  nickname: '',
  avatar: '',
  isVip: false,
  loginTime: ''
})

// 初始化
onMounted(async () => {
  await loadUserInfo()
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

// 退出登录
async function handleLogout() {
  if (confirm('确定要退出登录吗？')) {
    await settingStore.set('isAuthenticated', false)
    // TODO: 触发重新登录流程
    window.location.reload()
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
  </div>
</template>
<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { User, LogOut, UserCircle } from 'lucide-vue-next'
import { useAuthStore } from '@/store/authStore'
import { useToast } from '@/composables/useToast'

const authStore = useAuthStore()
const router = useRouter()
const { showToast } = useToast()

const user = authStore.user
const isLoggedIn = authStore.isLoggedIn
const showConfirmModal = ref(false)

function openConfirmLogout() {
  showConfirmModal.value = true
}

function confirmLogout() {
  authStore.logout()
  showConfirmModal.value = false
  showToast('已退出登录', 'success')
  setTimeout(() => {
    router.push('/login')
  }, 500)
}
</script>

<template>
  <div class="space-y-4">
    <!-- 用户信息卡片 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium flex items-center gap-2 mb-4">
          <User :size="16" />
          账户信息
        </h2>

        <div v-if="isLoggedIn" class="space-y-4">
          <!-- 用户头像和基本信息 -->
          <div class="flex items-center gap-4 p-3 bg-base-300 rounded-lg">
            <div class="flex-shrink-0">
              <div v-if="user?.avatar" class="w-16 h-16 rounded-2xl overflow-hidden ring-1 ring-base-300">
                <img :src="user.avatar" :alt="user.nickname" class="w-full h-full object-cover" />
              </div>
              <div v-else class="w-16 h-16 rounded-2xl bg-base-100 flex items-center justify-center ring-1 ring-base-300">
                <UserCircle :size="40" class="text-base-content/40" />
              </div>
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-base-content truncate">
                {{ user?.nickname || user?.name || '未设置昵称' }}
              </h3>
              <p class="text-xs text-base-content/60 mt-1">
                ID: {{ user?.id || '未知' }}
              </p>
            </div>
          </div>

          <!-- 用户昵称/用户名 -->
          <div class="flex items-center justify-between p-3 bg-base-100 rounded-lg">
            <span class="text-sm text-base-content/70">昵称</span>
            <span class="text-sm font-medium text-base-content">{{ user?.nickname || user?.name || '-' }}</span>
          </div>

          <!-- 登录状态 -->
          <div class="flex items-center justify-between p-3 bg-base-100 rounded-lg">
            <span class="text-sm text-base-content/70">登录状态</span>
            <span class="text-xs px-2 py-1 bg-success/10 text-success rounded-full">
              已登录
            </span>
          </div>

          <!-- 退出登录按钮 -->
          <div class="pt-2">
            <button
              @click="openConfirmLogout"
              class="btn btn-error btn-sm w-full gap-2"
            >
              <LogOut :size="14" />
              退出登录
            </button>
          </div>
        </div>

        <div v-else class="text-center py-8">
          <UserCircle :size="48" class="mx-auto text-base-content/30 mb-3" />
          <p class="text-sm text-base-content/50 mb-4">未登录</p>
          <button
            @click="() => router.push('/login')"
            class="btn btn-primary btn-sm gap-2"
          >
            去登录
          </button>
        </div>
      </div>
    </div>

    <!-- 退出登录确认 Modal -->
    <div v-if="showConfirmModal" class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">确认退出登录</h3>
        <p class="py-4 text-base-content/70">确定要退出当前账号吗？</p>
        <div class="modal-action">
          <button @click="showConfirmModal = false" class="btn btn-ghost btn-sm">
            取消
          </button>
          <button @click="confirmLogout" class="btn btn-error btn-sm">
            确认退出
          </button>
        </div>
      </div>
      <div class="modal-backdrop" @click="showConfirmModal = false"></div>
    </div>
  </div>
</template>

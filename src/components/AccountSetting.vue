<script setup>
import { ref, onMounted } from 'vue'
import { User, Mail, Camera, Edit2, Save } from 'lucide-vue-next'
import { useSettingStore } from '../store/settingStore'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'

const settingStore = useSettingStore()

// 用户信息
const userInfo = ref({
  nickname: '',
  email: '',
  avatar: ''
})

const isEditing = ref(false)
const editForm = ref({
  nickname: '',
  email: ''
})

const avatarPreview = ref('')

// 初始化
onMounted(async () => {
  await loadUserInfo()
})

// 加载用户信息
async function loadUserInfo() {
  try {
    const saved = await settingStore.get('userInfo', {
      nickname: '用户',
      email: 'user@example.com',
      avatar: ''
    })
    userInfo.value = saved || {
      nickname: '用户',
      email: 'user@example.com',
      avatar: ''
    }
    editForm.value = {
      nickname: saved?.nickname || '用户',
      email: saved?.email || 'user@example.com'
    }
    avatarPreview.value = saved?.avatar || ''
  } catch (error) {
    console.error('Failed to load user info:', error)
    // 设置默认值
    userInfo.value = {
      nickname: '用户',
      email: 'user@example.com',
      avatar: ''
    }
    editForm.value = {
      nickname: '用户',
      email: 'user@example.com'
    }
  }
}

// 开始编辑
function startEdit() {
  isEditing.value = true
  editForm.value = {
    nickname: userInfo.value?.nickname || '用户',
    email: userInfo.value?.email || 'user@example.com'
  }
}

// 取消编辑
function cancelEdit() {
  isEditing.value = false
  editForm.value = {
    nickname: userInfo.value?.nickname || '用户',
    email: userInfo.value?.email || 'user@example.com'
  }
}

// 保存用户信息
async function saveUserInfo() {
  try {
    const updated = {
      ...userInfo.value,
      nickname: editForm.value.nickname,
      email: editForm.value.email,
      avatar: avatarPreview.value
    }
    await settingStore.set('userInfo', updated)
    userInfo.value = updated
    isEditing.value = false
  } catch (error) {
    console.error('Failed to save user info:', error)
  }
}

// 选择头像
function selectAvatar() {
  // TODO: 实现头像选择功能
  console.log('选择头像')
}
</script>

<template>
  <div class="space-y-4">
    <!-- 账户信息 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between mb-4">
          <h2 class="card-title text-sm font-medium">
            <User :size="16" />
            账户信息
          </h2>
          <Button v-if="!isEditing" variant="ghost" size="sm" @click="startEdit">
            <Edit2 :size="12" />
            编辑
          </Button>
        </div>

        <div class="flex items-start gap-4">
          <!-- 头像 -->
          <div class="relative flex-shrink-0">
            <div class="w-20 h-20 rounded-full bg-primary/10 flex items-center justify-center overflow-hidden">
              <img v-if="avatarPreview" :src="avatarPreview" alt="头像" class="w-full h-full object-cover" />
              <User v-else :size="32" class="text-primary/40" />
            </div>
            <button
              @click="selectAvatar"
              class="absolute -bottom-1 -right-1 w-7 h-7 bg-primary text-primary-content rounded-full flex items-center justify-center hover:bg-primary/90 transition-colors"
            >
              <Camera :size="12" />
            </button>
          </div>

          <!-- 用户信息表单 -->
          <div class="flex-1 space-y-3 min-w-0">
            <div class="form-control">
              <label class="label">
                <span class="label-text text-xs">昵称</span>
              </label>
              <Input
                v-if="isEditing"
                type="text"
                v-model="editForm.nickname"
                placeholder="请输入昵称"
                size="sm"
              />
              <div v-else class="text-sm font-medium truncate">{{ userInfo?.nickname || '用户' }}</div>
            </div>

            <div class="form-control">
              <label class="label">
                <span class="label-text text-xs">邮箱</span>
              </label>
              <Input
                v-if="isEditing"
                type="email"
                v-model="editForm.email"
                placeholder="请输入邮箱"
                size="sm"
              />
              <div class="flex items-center gap-1">
                <Mail :size="12" class="text-base-content/40" />
                <span class="text-sm text-base-content/60 truncate">{{ userInfo?.email || 'user@example.com' }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 编辑操作按钮 -->
        <div v-if="isEditing" class="flex gap-2 mt-4">
          <Button variant="primary" size="sm" @click="saveUserInfo">
            <Save :size="12" />
            保存
          </Button>
          <Button variant="ghost" size="sm" @click="cancelEdit">
            取消
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
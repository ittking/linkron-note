<script setup>
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useSettingStore } from '../store/settingStore'
import { Power, Folder, Image } from 'lucide-vue-next'
import Toggle from './ui/Toggle.vue'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'

const settingStore = useSettingStore()

// 开机启动
const autoStartEnabled = ref(false)

// 笔记图片最大展示数
const noteImageMaxCount = ref(4)

// 工作目录
const workDirectory = ref('')
const workDirectoryStatus = ref(null)

// 初始化
onMounted(async () => {
  await loadAutoStartStatus()
  await loadWorkDirectory()
  await loadNoteImageMaxCount()
})

// 加载开机启动状态
async function loadAutoStartStatus() {
  try {
    autoStartEnabled.value = await invoke('is_autostart_enabled')
  } catch (error) {
    console.error('Failed to load autostart status:', error)
  }
}

// 监听开机启动状态变化
watch(autoStartEnabled, async (newValue) => {
  try {
    await invoke('set_autostart', { enable: newValue })
  } catch (error) {
    console.error('Failed to toggle autostart:', error)
    // 恢复状态
    await loadAutoStartStatus()
  }
})

// 加载工作目录
async function loadWorkDirectory() {
  try {
    workDirectory.value = await settingStore.get('workDirectory', '')
  } catch (error) {
    console.error('Failed to load work directory:', error)
  }
}

// 选择工作目录
async function selectWorkDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择工作目录'
    })
    
    if (selected) {
      workDirectory.value = selected
    }
  } catch (error) {
    console.error('Failed to select directory:', error)
  }
}

// 保存工作目录
async function saveWorkDirectory() {
  workDirectoryStatus.value = null

  try {
    if (workDirectory.value.trim()) {
      // 检查目录是否存在
      const exists = await invoke('check_directory_exists', { path: workDirectory.value.trim() })

      if (!exists) {
        // 创建目录
        await invoke('create_directory', { path: workDirectory.value.trim() })
        workDirectoryStatus.value = {
          type: 'success',
          message: '工作目录已创建并保存'
        }
      } else {
        workDirectoryStatus.value = {
          type: 'success',
          message: '工作目录已保存'
        }
      }

      await settingStore.set('workDirectory', workDirectory.value.trim())
    } else {
      // 清空工作目录
      await settingStore.set('workDirectory', '')
      workDirectoryStatus.value = {
        type: 'success',
        message: '已恢复默认工作目录'
      }
    }
  } catch (error) {
    console.error('Failed to save work directory:', error)
    workDirectoryStatus.value = {
      type: 'error',
      message: '保存失败: ' + error.message
    }
  }
}

// 加载笔记图片最大展示数
async function loadNoteImageMaxCount() {
  try {
    const savedValue = await settingStore.get('noteImageMaxCount', 4)
    noteImageMaxCount.value = Number(savedValue)
  } catch (error) {
    console.error('Failed to load note image max count:', error)
  }
}

// 监听笔记图片最大展示数变化
watch(noteImageMaxCount, async (newValue) => {
  const count = Number(newValue)
  if (count >= 1 && count <= 20) {
    try {
      await settingStore.set('noteImageMaxCount', count)
    } catch (error) {
      console.error('Failed to save note image max count:', error)
    }
  }
})

</script>

<template>
  <div class="space-y-4">
    <!-- 开机启动 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4 ">
        <h2 class="card-title text-sm font-medium">
          <Power :size="16" />
          开机启动
        </h2>
        <div class="form-control">
          <label class="label cursor-pointer flex justify-between gap-4">
            <span class="label-text">开机自动启动</span>
            <Toggle v-model="autoStartEnabled" size="sm" />
          </label>
        </div>
      </div>
    </div>

    <!-- 工作目录设置 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium">
          <Folder :size="16" />
          工作目录
        </h2>
        <div class="space-y-3">
          <div class="form-control">
            <label class="label">
              <span class="label-text text-xs">工作目录路径</span>
            </label>
            <Input type="text" v-model="workDirectory" placeholder="留空使用默认路径" size="sm" />
          </div>
          <div class="flex gap-2">
            <Button variant="primary" size="sm" block @click="saveWorkDirectory">
              保存工作目录
            </Button>
            <Button variant="ghost" size="sm" @click="selectWorkDirectory">
              选择目录
            </Button>
          </div>
          <div v-if="workDirectoryStatus" :class="['text-xs', workDirectoryStatus.type === 'success' ? 'text-success' : 'text-error']">
            {{ workDirectoryStatus.message }}
          </div>
        </div>
      </div>
    </div>

    <!-- 笔记图片最大展示数 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <h2 class="card-title text-sm font-medium">
          <Image :size="16" />
          笔记图片
        </h2>
        <div class="form-control">
          <label class="label">
            <span class="label-text text-xs">笔记卡片图片列表最大展示数量 (1-20)</span>
          </label>
          <Input
            type="number"
            v-model.number="noteImageMaxCount"
            min="1"
            max="20"
            placeholder="默认为 4"
            size="sm"
          />
        </div>
      </div>
    </div>
  </div>
</template>
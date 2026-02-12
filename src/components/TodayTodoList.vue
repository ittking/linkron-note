<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Check, Clock } from 'lucide-vue-next'
import dayjs from 'dayjs'
import dayjsLocale from 'dayjs/locale/zh-cn'
import DateTimePicker from './ui/DateTimePicker.vue'
import { useWorkDirectory } from '@/composables/useWorkDirectory'

dayjs.locale(dayjsLocale)

const { getWorkDirectory } = useWorkDirectory('setting')

const todos = ref([])
const newTodoText = ref('')
const selectedReminderTime = ref('')
const loading = ref(false)

// 状态颜色映射
const STATUS_COLORS = {
  'todo': { bg: '#9CA3AF', border: '#6B7280' },
  'in-progress': { bg: '#3B82F6', border: '#2563EB' },
  'completed': { bg: '#10B981', border: '#059669' },
  'pending': { bg: '#F59E0B', border: '#D97706' },
  'cancelled': { bg: '#EF4444', border: '#DC2626' }
}

// 获取今日日期字符串
const today = dayjs().format('YYYY-MM-DD')

// 获取今日显示的日期信息
const todayDisplay = computed(() => {
  return {
    full: `今日 · ${dayjs().format('YYYY年M月D日')} · ${dayjs().format('dddd')}`
  }
})

// 从后端加载待办事项
async function loadTodos() {
  loading.value = true
  try {
    const workDirectory = await getWorkDirectory()
    // 调用新的后端接口，获取今日相关的待办事项
    const data = await invoke('get_today_todos', { 
      todayDate: today,
      workDirectory 
    })
    todos.value = data
  } catch (error) {
    console.error('加载待办事项失败:', error)
  } finally {
    loading.value = false
  }
}

// 获取状态颜色
function getStatusColor(status) {
  return STATUS_COLORS[status] || STATUS_COLORS['todo']
}

// 创建待办事项
async function createTodo() {
  if (!newTodoText.value.trim()) return

  try {
    const workDirectory = await getWorkDirectory()
    
    // 构建提醒配置
    let reminder = null
    if (selectedReminderTime.value) {
      reminder = JSON.stringify({
        type: 'onetime',
        repeat_time: selectedReminderTime.value
      })
    }

    await invoke('create_todo', {
      date: today,
      text: newTodoText.value,
      status: 'todo',
      reminder,
      workDirectory
    })

    newTodoText.value = ''
    selectedReminderTime.value = ''
    await loadTodos()
  } catch (error) {
    console.error('创建待办失败:', error)
  }
}

// 切换待办完成状态
async function toggleTodoStatus(todo) {
  const newStatus = todo.status === 'completed' ? 'todo' : 'completed'
  try {
    const workDirectory = await getWorkDirectory()
    await invoke('update_todo', {
      id: todo.id,
      text: todo.text,
      status: newStatus,
      reminder: todo.reminder ? JSON.stringify(todo.reminder) : null,
      workDirectory
    })
    await loadTodos()
  } catch (error) {
    console.error('更新待办状态失败:', error)
  }
}

// 删除待办事项
async function deleteTodo(todo) {
  try {
    const workDirectory = await getWorkDirectory()
    await invoke('delete_todo', {
      id: todo.id,
      workDirectory
    })
    await loadTodos()
  } catch (error) {
    console.error('删除待办失败:', error)
  }
}

// 从 todo 对象获取提醒时间
function getReminderTime(todo) {
  if (!todo.reminder) return null
  const reminder = todo.reminder
  return reminder.reminder_time || reminder.repeatTime || null
}

// 格式化提醒时间显示
function formatReminderTime(timeStr) {
  if (!timeStr) return ''
  const date = dayjs(timeStr)
  const dateStr = date.format('MM/DD')
  const timeStr2 = date.format('HH:mm')
  return `${dateStr} ${timeStr2}`
}

// 后端已排序，直接使用
const sortedTodos = computed(() => todos.value)

onMounted(() => {
  loadTodos()
})
</script>

<template>
  <div class="today-todo-list h-full flex flex-col bg-base-100">
    <!-- 顶部：今日日期 -->
    <div class="px-6 py-3 border-b border-base-200">
      <div class="text-center text-sm text-base-content/80 font-medium">
        {{ todayDisplay.full }}
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="px-6 py-4 border-b border-base-200">
      <textarea
        v-model="newTodoText"
        placeholder="输入待办事项..."
        class="w-full px-4 py-2.5 border border-base-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent text-sm resize-none"
        rows="2"
      ></textarea>
      <div class="flex items-center justify-between mt-3">
        <DateTimePicker
          v-model="selectedReminderTime"
          mode="datetime"
          :min="dayjs().format('YYYY-MM-DDTHH:mm')"
          placeholder="设置提醒"
        />
        <button
          @click="createTodo"
          :disabled="!newTodoText.trim()"
          class="px-4 py-1.5 bg-primary text-primary-content rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm whitespace-nowrap"
        >
          创建
        </button>
      </div>
    </div>

    <!-- 待办列表 -->
    <div class="flex-1 overflow-auto px-6 py-4">
      <div v-if="loading" class="flex justify-center py-8">
        <span class="loading loading-spinner text-primary"></span>
      </div>

      <div v-else-if="sortedTodos.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/40 text-center py-12">
        <Check :size="48" class="mb-3 opacity-50" />
        <div class="text-sm">今日暂无待办事项</div>
      </div>

      <div v-else class="space-y-3">
        <div
          v-for="todo in sortedTodos"
          :key="todo.id"
          class="flex items-center gap-3 p-3 bg-base-100 border border-base-200 rounded-lg hover:border-base-300 transition-colors"
        >
          <!-- 圆形 Checkbox -->
          <button
            @click="toggleTodoStatus(todo)"
            class="flex-shrink-0 w-5 h-5 rounded-full border-2 flex items-center justify-center transition-all"
            :class="{
              'bg-opacity-100': todo.status === 'completed',
              'border-base-300 hover:border-primary': todo.status !== 'completed'
            }"
            :style="todo.status !== 'completed' ? {
              borderColor: getStatusColor(todo.status).border
            } : {}"
          >
            <Check v-if="todo.status === 'completed'" :size="12" class="text-white" />
            <div v-else class="w-2 h-2 rounded-full transition-all" :style="{ backgroundColor: getStatusColor(todo.status).bg }"></div>
          </button>

          <!-- 待办内容 -->
          <div class="flex-1 min-w-0">
            <div
              class="text-sm"
              :class="{
                'line-through opacity-60': todo.status === 'completed' || todo.status === 'cancelled'
              }"
            >
              {{ todo.text }}
            </div>

            <!-- 提醒时间 -->
            <div v-if="getReminderTime(todo)" class="flex items-center gap-1 mt-1 text-xs text-base-content/50">
              <Clock :size="12" />
              {{ formatReminderTime(getReminderTime(todo)) }}
            </div>
          </div>

          <!-- 删除按钮 -->
          <button
            @click="deleteTodo(todo)"
            class="flex-shrink-0 p-1 text-base-content/40 hover:text-error hover:bg-error/10 rounded transition-colors"
            title="删除"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.no-scrollbar {
  scrollbar-width: none;
}

.no-scrollbar::-webkit-scrollbar {
  display: none;
}
</style>
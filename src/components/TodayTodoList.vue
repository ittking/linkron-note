<script setup>
import { ref, computed } from 'vue'
import { Check, Clock, X } from 'lucide-vue-next'
import dayjs from 'dayjs'
import dayjsLocale from 'dayjs/locale/zh-cn'
import DateTimePicker from './ui/DateTimePicker.vue'

dayjs.locale(dayjsLocale)

const props = defineProps({
  todos: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['create', 'update', 'delete', 'toggle-status'])

const newTodoText = ref('')
const selectedReminderTime = ref('')

// 获取今日显示的日期信息
const todayDisplay = computed(() => {
  return {
    full: `今日 · ${dayjs().format('YYYY年M月D日')} · ${dayjs().format('dddd')}`
  }
})

// 状态颜色映射
const STATUS_COLORS = {
  'todo': '#6B7280',
  'in-progress': '#3B82F6',
  'completed': '#10B981',
  'pending': '#F59E0B',
  'cancelled': '#EF4444'
}

// 从 todo 对象获取提醒时间
function getReminderTime(todo) {
  if (!todo.reminder) return null
  const reminder = todo.reminder
  return reminder.repeat_time || reminder.repeatTime || null
}

// 格式化提醒时间显示
function formatReminderTime(timeStr) {
  if (!timeStr) return ''
  const date = dayjs(timeStr)
  const dateStr = date.format('MM/DD')
  const timeStr2 = date.format('HH:mm')
  return `${dateStr} ${timeStr2}`
}

// 创建待办事项
function createTodo() {
  if (!newTodoText.value.trim()) return

  // 构建提醒配置
  let reminder = null
  if (selectedReminderTime.value) {
    reminder = JSON.stringify({
      type: 'onetime',
      repeatTime: selectedReminderTime.value
    })
  }

  emit('create', {
    date: dayjs().format('YYYY-MM-DD'),
    text: newTodoText.value,
    status: 'todo',
    reminder
  })

  newTodoText.value = ''
  selectedReminderTime.value = ''
}

// 切换待办完成状态
function toggleTodoStatus(todo) {
  emit('toggle-status', todo)
}

// 删除待办事项
function deleteTodo(todo) {
  emit('delete', todo.id)
}

// 更新定时时间
function updateReminderTime(todo, value) {
  let reminder = null
  if (value) {
    reminder = JSON.stringify({
      type: 'onetime',
      repeatTime: value
    })
  }

  emit('update', {
    id: todo.id,
    text: todo.text,
    status: todo.status,
    reminder
  })
}

// 后端已排序，直接使用
const sortedTodos = computed(() => props.todos)
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
    <div class="p-4">
      <div class="flex items-center gap-2">
        <input
          v-model="newTodoText"
          @keyup.enter="createTodo"
          type="text"
          placeholder="输入待办事项，按回车创建..."
          class="flex-1 px-3 py-2 border border-base-200 rounded-md focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent text-sm h-[34px]"
        />
        <DateTimePicker
          v-model="selectedReminderTime"
          mode="datetime"
          :min="dayjs().format('YYYY-MM-DDTHH:mm')"
          :clearable="true"
        >
          <template #default="{ toggle }">
            <button
              @click="toggle"
              class="rounded-md transition-colors flex-shrink-0 border border-base-200 h-[34px] w-[34px] flex items-center justify-center"
              :class="{
                'text-primary bg-primary/10 border-primary/30': !!selectedReminderTime,
                'text-base-content/40 hover:text-primary hover:bg-primary/5 hover:border-primary/30': !selectedReminderTime
              }"
              title="设置提醒时间"
            >
              <Clock :size="16" />
            </button>
          </template>
        </DateTimePicker>
      </div>
    </div>

    <!-- 待办列表 -->
    <div class="flex-1 overflow-auto p-4 pt-0">
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
          class="group flex items-start gap-3 p-3 bg-primary/5 rounded-lg"
        >
          <!-- 圆形 Checkbox -->
          <button
            @click="toggleTodoStatus(todo)"
            class="flex-shrink-0 w-4 h-4 rounded-full border-2 flex items-center justify-center transition-all"
            :style="{
              borderColor: todo.status === 'completed' ? 'rgba(16, 185, 129, 0.5)' : STATUS_COLORS[todo.status],
              backgroundColor: 'transparent'
            }"
          >
            <div v-if="todo.status === 'completed'" class="w-1.5 h-1.5 rounded-full" style="background-color: #10B981"></div>
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
            <div class="mt-3">
              <DateTimePicker
                :model-value="getReminderTime(todo)"
                @update:model-value="(value) => updateReminderTime(todo, value)"
                mode="datetime"
                :min="dayjs().format('YYYY-MM-DDTHH:mm')"
                :clearable="true"
              >
                <template #default="{ toggle, hasValue }">
                  <div 
                    @click="toggle"
                    class="flex items-center gap-1 text-xs cursor-pointer hover:text-primary transition-colors"
                    :class="{
                      'text-base-content/50': !hasValue,
                      'text-base-content/70': hasValue
                    }"
                  >
                    <Clock :size="12" />
                    {{ hasValue ? formatReminderTime(getReminderTime(todo)) : '今日' }}
                  </div>
                </template>
              </DateTimePicker>
            </div>
          </div>

          <!-- 删除按钮 -->
          <button
            @click="deleteTodo(todo)"
            class="flex-shrink-0 p-1 text-base-content/40 hover:text-error hover:bg-error/10 rounded transition-colors opacity-0 group-hover:opacity-100"
            title="删除"
          >
            <X :size="14" />
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
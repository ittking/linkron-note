<script setup>
import { ref, computed, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { Check, Clock, X, MoreHorizontal, Repeat } from 'lucide-vue-next'
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

const emit = defineEmits(['create', 'update', 'delete', 'toggle-status', 'date-change', 'open-edit'])

const newTodoText = ref('')
const selectedReminderTime = ref('')

// 实时时钟
const currentTime = ref('')
let timer = null

// 更新当前时间
function updateCurrentTime() {
  currentTime.value = dayjs().format('HH:mm:ss')
}

onMounted(() => {
  updateCurrentTime()
  timer = setInterval(updateCurrentTime, 1000)
})

onBeforeUnmount(() => {
  if (timer) {
    clearInterval(timer)
  }
})

// 编辑状态
const editingTodoId = ref(null)
const editingTodoText = ref('')

// 选中的日期
const selectedDate = ref(dayjs().format('YYYY-MM-DD'))

// 获取显示的日期信息
const displayDateInfo = computed(() => {
  const date = dayjs(selectedDate.value)
  const today = dayjs()
  const isToday = date.isSame(today, 'day')
  const prefix = isToday ? '今日' : ''
  const fullDate = `${date.format('YYYY年M月D日')}`
  const weekday = date.format('dddd')
  return {
    full: isToday ? `${prefix} · ${fullDate} · ${weekday}` : `${fullDate} · ${weekday}`
  }
})

// 监听日期变化，重新加载待办事项
watch(selectedDate, () => {
  emit('date-change', selectedDate.value)
})

// 辅助函数：序列化 reminder 对象
function stringifyReminder(reminderObj) {
  if (!reminderObj) return null
  try {
    return JSON.stringify(reminderObj)
  } catch {
    return null
  }
}

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
  // 重复提醒：返回 HH:mm 格式（用于 time 模式）
  if (reminder.type === 'repeat' && reminder.repeatTime) {
    return dayjs(reminder.repeatTime).format('HH:mm')
  }
  // 一次性提醒：返回完整日期时间格式（用于 datetime 模式）
  return reminder.repeatTime || null
}

// 格式化提醒时间显示
function formatReminderTime(timeStr, todo) {
  if (!timeStr) return ''
  if (!todo.reminder) return ''

  const reminder = todo.reminder
  if (reminder.type === 'repeat') {
    // 重复提醒：根据重复类型显示不同的文本
    // 使用完整的 repeatTime 来解析时间
    const fullTime = reminder.repeatTime || timeStr
    const dateObj = dayjs(fullTime)
    const time = dateObj.isValid() ? dateObj.format('HH:mm') : ''
    const rule = reminder.repeatRule

    if (rule === 'day') {
      const interval = reminder.repeatInterval || 1
      return interval === 1 ? `每天: ${time}` : `每${interval}天: ${time}`
    } else if (rule === 'weekday') {
      const weekdays = reminder.repeatDayOfWeek
      const weekdayArray = Array.isArray(weekdays) ? weekdays : [weekdays]
      const weekdayNames = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
      const selectedDays = weekdayArray.map(day => weekdayNames[day]).join('、')
      return `每周: ${selectedDays} ${time}`
    } else if (rule === 'month') {
      const days = reminder.repeatDayOfMonth
      const dayArray = Array.isArray(days) ? days : [days]
      const selectedDays = dayArray.join('、')
      return `每月: ${selectedDays}日 ${time}`
    } else if (rule === 'year') {
      const month = reminder.repeatMonth || 1
      const day = reminder.repeatDayOfMonth || 1
      return `每年: ${month}月${day}日 ${time}`
    }
    return time
  } else {
    // 一次性提醒：显示完整日期时间
    const date = dayjs(timeStr)
    if (!date.isValid()) return ''
    const dateStr = date.format('MM/DD')
    const timeStr2 = date.format('HH:mm')
    return `${dateStr} ${timeStr2}`
  }
}

// 创建待办事项
function createTodo() {
  if (!newTodoText.value.trim()) return

  // 构建提醒配置
  let reminder = null
  if (selectedReminderTime.value) {
    reminder = JSON.stringify({
      type: 'onetime',
      repeatTime: selectedReminderTime.value // 一次性提醒：完整日期时间
    })
  }

  emit('create', {
    date: selectedDate.value,
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

function updateReminderTime(todo, value) {
  let reminder = null
  
  if (value && value.trim() !== '') {
    const originalReminderType = todo.reminder?.type || 'onetime'
    let timeValue

    if (originalReminderType === 'onetime') {
      const selectedDateObj = dayjs(selectedDate.value)
      const timeOnly = value.includes('T') ? value.split('T')[1] : value
      timeValue = `${selectedDateObj.format('YYYY-MM-DD')}T${timeOnly}`
    } else {
      const timeOnly = value.includes('T') ? value.split('T')[1] : value
      const selectedDateObj = dayjs(selectedDate.value)
      timeValue = timeOnly ? `${selectedDateObj.format('YYYY-MM-DD')}T${timeOnly}` : ''
    }

    const reminderObj = {
      type: originalReminderType,
      repeatTime: timeValue,
      ...(originalReminderType === 'repeat' ? {
        repeatRule: todo.reminder?.repeatRule || 'day',
        repeatInterval: todo.reminder?.repeatInterval || 1,
        repeatDayOfWeek: todo.reminder?.repeatDayOfWeek,
        repeatDayOfMonth: todo.reminder?.repeatDayOfMonth,
        repeatMonth: todo.reminder?.repeatMonth
      } : {})
    }

    reminder = JSON.stringify(reminderObj)
  }

  emit('update', {
    id: todo.id,
    text: todo.text,
    status: todo.status,
    reminder,
    date: selectedDate.value
  })
}

// 开始编辑
function startEditing(todo) {
  editingTodoId.value = todo.id
  editingTodoText.value = todo.text
  // 使用 nextTick 确保 DOM 更新后再聚焦
  nextTick(() => {
    const inputEl = document.getElementById(`todo-input-${todo.id}`)
    if (inputEl) {
      inputEl.focus()
      inputEl.select()
    }
  })
}

// 取消编辑
function cancelEditing() {
  editingTodoId.value = null
  editingTodoText.value = ''
}

// 更新待办文本
function updateTodoText(todo) {
  if (!editingTodoText.value.trim()) {
    cancelEditing()
    return
  }

  emit('update', {
    id: todo.id,
    text: editingTodoText.value.trim(),
    status: todo.status,
    reminder: stringifyReminder(todo.reminder)
  })

  cancelEditing()
}

// 打开编辑对话框
function openEditDialog(todo) {
  emit('open-edit', selectedDate.value, todo)
}

// 后端已排序，直接使用
const sortedTodos = computed(() => props.todos)
</script>

<template>
  <div class="today-todo-list h-full flex flex-col bg-base-100 max-w-200 mx-auto pb-2">
    <!-- 顶部：日期选择 -->
    <div class="px-6 py-3 border-b border-base-200">
      <div class="flex items-center justify-between">
        <DateTimePicker v-model="selectedDate" mode="date" placeholder="选择日期">
          <template #default="{ toggle }">
            <div @click="toggle"
              class="text-sm text-base-content/80 font-medium cursor-pointer hover:text-primary transition-colors">
              {{ displayDateInfo.full }}
            </div>
          </template>
        </DateTimePicker>
        <div class="flex items-center gap-1 text-sm text-base-content/60">
          <Clock :size="14" />
          <span>{{ currentTime }}</span>
        </div>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="p-4">
      <div class="flex items-center gap-2">
        <input v-model="newTodoText" @keypress.enter="createTodo" type="text" placeholder="输入待办事项，按回车创建..."
          class="flex-1 px-3 py-2 border border-base-200 rounded-md focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent text-sm h-[34px]" />
        <DateTimePicker v-model="selectedReminderTime" mode="datetime" :min="dayjs().format('YYYY-MM-DDTHH:mm')"
          :clearable="true">
          <template #default="{ toggle }">
            <button @click="toggle"
              class="rounded-md transition-colors flex-shrink-0 border border-base-200 h-[34px] w-[34px] flex items-center justify-center"
              :class="{
                'text-primary bg-primary/10 border-primary/30': !!selectedReminderTime,
                'text-base-content/40 hover:text-primary hover:bg-primary/5 hover:border-primary/30': !selectedReminderTime
              }" title="设置提醒时间">
              <Clock :size="16" />
            </button>
          </template>
        </DateTimePicker>
      </div>
    </div>

    <!-- 待办列表 -->
    <div class="flex-1 overflow-auto p-4 pt-0 no-scrollbar">
      <div v-if="loading" class="flex justify-center py-8">
        <span class="loading loading-spinner text-primary"></span>
      </div>

      <div v-else-if="sortedTodos.length === 0"
        class="flex flex-col items-center justify-center h-full text-base-content/40 text-center py-12">
        <Check :size="48" class="mb-3 opacity-50" />
        <div class="text-sm">今日暂无待办事项</div>
      </div>

      <div v-else class="space-y-3">
        <div v-for="todo in sortedTodos" :key="todo.id"
          class="group flex items-start gap-3 p-3 bg-primary/5 rounded-lg">
          <!-- 圆形 Checkbox -->
          <button @click="toggleTodoStatus(todo)"
            class="flex-shrink-0 w-4 h-4 rounded-full border-2 flex items-center justify-center transition-all" :style="{
              borderColor: todo.status === 'completed' ? 'rgba(16, 185, 129, 0.5)' :
                todo.status === 'cancelled' ? 'rgba(239, 68, 68, 0.5)' :
                  todo.status === 'pending' ? 'rgba(245, 158, 11, 0.5)' :
                    STATUS_COLORS[todo.status],
              backgroundColor: 'transparent'
            }">
            <div v-if="todo.status === 'completed'" class="w-1.5 h-1.5 rounded-full" style="background-color: #10B981">
            </div>
            <div v-else-if="todo.status === 'cancelled'" class="w-1.5 h-1.5 rounded-full"
              style="background-color: #EF4444">
            </div>
            <div v-else-if="todo.status === 'pending'" class="w-1.5 h-1.5 rounded-full"
              style="background-color: #F59E0B">
            </div>
          </button>

          <!-- 待办内容 -->
          <div class="flex-1 min-w-0 line-height-1">
            <!-- 编辑模式 -->
            <input v-if="editingTodoId === todo.id" :id="`todo-input-${todo.id}`" v-model="editingTodoText"
              @blur="updateTodoText(todo)" @keyup.enter="updateTodoText(todo)" @keyup.esc="cancelEditing"
              class="w-full text-sm bg-transparent border-none focus:outline-none focus:ring-0 p-0 !m-0" />
            <!-- 显示模式 -->
            <div v-else @click="todo.status !== 'completed' && todo.status !== 'cancelled' && startEditing(todo)"
              class="text-sm cursor-pointer" :class="{
                'line-through opacity-60': todo.status === 'completed' || todo.status === 'cancelled' || todo.status === 'pending'
              }">
              {{ todo.text }}
            </div>

            <!-- 提醒时间 -->
            <div class="mt-3">
              <DateTimePicker :model-value="getReminderTime(todo)"
                @update:model-value="(value) => updateReminderTime(todo, value)"
                :mode="todo.reminder?.type === 'repeat' ? 'time' : 'datetime'" :min="dayjs().format('YYYY-MM-DDTHH:mm')"
                :clearable="true">
                <template #default="{ toggle, hasValue }">
                  <div class="inline-block">
                    <div @click="toggle"
                      class="flex items-start gap-1 text-xs cursor-pointer hover:text-primary transition-colors leading-tight"
                      :class="{
                        'text-base-content/50': !hasValue,
                        'text-base-content/70': hasValue
                      }">
                      <span>
                        <Repeat v-if="hasValue && todo.reminder?.type === 'repeat'" :size="14" />
                        <Clock v-else :size="14" />
                      </span>
                      <span v-if="hasValue">{{ formatReminderTime(getReminderTime(todo), todo) }}</span>
                      <span v-if="!hasValue">今日</span>
                    </div>
                  </div>
                </template>
              </DateTimePicker>
            </div>
          </div>

          <!-- 更多操作按钮 -->
          <button @click="openEditDialog(todo)"
            class="flex-shrink-0 p-1 text-base-content/40 hover:text-primary hover:bg-primary/10 rounded transition-colors opacity-0 group-hover:opacity-100"
            title="编辑">
            <MoreHorizontal :size="14" />
          </button>

          <!-- 删除按钮 -->
          <button @click="deleteTodo(todo)"
            class="flex-shrink-0 p-1 text-base-content/40 hover:text-error hover:bg-error/10 rounded transition-colors opacity-0 group-hover:opacity-100"
            title="删除">
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
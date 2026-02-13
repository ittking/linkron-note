<script setup>
import { ref, computed, nextTick, watch } from 'vue'
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
  if (reminder.reminder_type === 'repeat' && reminder.repeat_time) {
    return dayjs(reminder.repeat_time).format('HH:mm')
  }
  // 一次性提醒：返回完整日期时间格式（用于 datetime 模式）
  return reminder.repeat_time || null
}

// 格式化提醒时间显示
function formatReminderTime(timeStr, todo) {
  if (!timeStr) return ''
  if (!todo.reminder) return ''

  const reminder = todo.reminder
  if (reminder.reminder_type === 'repeat') {
    // 重复提醒：根据重复类型显示不同的文本
    // 使用完整的 repeat_time 来解析时间
    const fullTime = reminder.repeat_time || timeStr
    const dateObj = dayjs(fullTime)
    const time = dateObj.isValid() ? dateObj.format('HH:mm') : ''
    const rule = reminder.repeat_rule

    if (rule === 'day') {
      const interval = reminder.repeat_interval || 1
      return interval === 1 ? `每天: ${time}` : `每${interval}天: ${time}`
    } else if (rule === 'weekday') {
      const weekdays = reminder.repeat_day_of_week
      const weekdayArray = Array.isArray(weekdays) ? weekdays : [weekdays]
      const weekdayNames = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
      const selectedDays = weekdayArray.map(day => weekdayNames[day]).join('、')
      return `每周: ${selectedDays} ${time}`
    } else if (rule === 'month') {
      const days = reminder.repeat_day_of_month
      const dayArray = Array.isArray(days) ? days : [days]
      const selectedDays = dayArray.join('、')
      return `每月: ${selectedDays}日 ${time}`
    } else if (rule === 'year') {
      const month = reminder.repeat_month || 1
      const day = reminder.repeat_day_of_month || 1
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
      reminder_type: 'onetime',
      repeat_time: selectedReminderTime.value // 一次性提醒：完整日期时间
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

// 更新定时时间
function updateReminderTime(todo, value) {
  let reminder = null
  if (value) {
    // 获取原有的提醒类型
    const originalReminderType = todo.reminder?.reminder_type || 'onetime'
    let timeValue

    if (originalReminderType === 'onetime') {
      // 一次性提醒：保持完整的日期时间格式
      timeValue = value
    } else {
      // 重复提醒：将 HH:mm 转换为今日的完整日期时间
      const timeOnly = value || ''
      timeValue = timeOnly ? `${dayjs().format('YYYY-MM-DD')}T${timeOnly}` : ''
    }

    reminder = JSON.stringify({
      reminder_type: originalReminderType,
      repeat_time: timeValue,
      // 保持原有的重复规则
      repeat_rule: todo.reminder?.repeat_rule,
      repeat_interval: todo.reminder?.repeat_interval,
      repeat_day_of_week: todo.reminder?.repeat_day_of_week,
      repeat_day_of_month: todo.reminder?.repeat_day_of_month,
      repeat_month: todo.reminder?.repeat_month
    })
  }

  emit('update', {
    id: todo.id,
    text: todo.text,
    status: todo.status,
    reminder
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

  // 保持原有的提醒配置不变
  let reminder = null
  if (todo.reminder) {
    reminder = JSON.stringify(todo.reminder)
  }

  emit('update', {
    id: todo.id,
    text: editingTodoText.value.trim(),
    status: todo.status,
    reminder
  })

  cancelEditing()
}

// 打开编辑对话框
function openEditDialog(todo) {
  emit('open-edit', selectedDate.value, todo)
}

// 后端已排序，直接使用
const sortedTodos = computed(() => {
  return [...props.todos].sort((a, b) => {
    // 先按状态排序：已完成（completed、cancelled）在后
    const aCompleted = a.status === 'completed' || a.status === 'cancelled'
    const bCompleted = b.status === 'completed' || b.status === 'cancelled'

    if (aCompleted && !bCompleted) return 1  // a完成，b未完成，b在前
    if (!aCompleted && bCompleted) return -1  // a未完成，b完成，a在前

    // 同状态按创建时间排序（新的在前）
    const aTime = new Date(a.created_at).getTime()
    const bTime = new Date(b.created_at).getTime()
    return bTime - aTime
  })
})
</script>

<template>
  <div class="today-todo-list h-full flex flex-col bg-base-100 max-w-200 mx-auto pb-2">
    <!-- 顶部：日期选择 -->
    <div class="px-6 py-3 border-b border-base-200">
      <div class="flex items-center justify-center">
        <DateTimePicker v-model="selectedDate" mode="date" placeholder="选择日期">
          <template #default="{ toggle }">
            <div @click="toggle"
              class="text-sm text-base-content/80 font-medium cursor-pointer hover:text-primary transition-colors">
              {{ displayDateInfo.full }}
            </div>
          </template>
        </DateTimePicker>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="p-4">
      <div class="flex items-center gap-2">
        <input v-model="newTodoText" @keyup.enter="createTodo" type="text" placeholder="输入待办事项，按回车创建..."
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
              borderColor: todo.status === 'completed' ? 'rgba(16, 185, 129, 0.5)' : STATUS_COLORS[todo.status],
              backgroundColor: 'transparent'
            }">
            <div v-if="todo.status === 'completed'" class="w-1.5 h-1.5 rounded-full" style="background-color: #10B981">
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
                'line-through opacity-60': todo.status === 'completed' || todo.status === 'cancelled'
              }">
              {{ todo.text }}
            </div>

            <!-- 提醒时间 -->
            <div class="mt-3">
              <DateTimePicker :model-value="getReminderTime(todo)"
                @update:model-value="(value) => updateReminderTime(todo, value)"
                :mode="todo.reminder?.reminder_type === 'repeat' ? 'time' : 'datetime'"
                :min="dayjs().format('YYYY-MM-DDTHH:mm')" :clearable="true">
                <template #default="{ toggle, hasValue }">
                  <div @click="toggle"
                    class="flex items-start gap-1 text-xs cursor-pointer hover:text-primary transition-colors" :class="{
                      'text-base-content/50': !hasValue,
                      'text-base-content/70': hasValue
                    }">
                    <Repeat v-if="hasValue && todo.reminder?.reminder_type === 'repeat'" :size="12" />
                    <Clock v-else :size="12" />
                    <span v-if="hasValue">{{ formatReminderTime(getReminderTime(todo), todo) }}</span>
                    <span v-if="!hasValue">今日</span>
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
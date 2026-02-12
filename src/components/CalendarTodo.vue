<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { ChevronLeft, ChevronRight, Plus, X } from 'lucide-vue-next'
import dayjs from 'dayjs'
import Button from './ui/Button.vue'
import Toggle from './ui/Toggle.vue'
import Input from './ui/Input.vue'

dayjs.locale('zh-cn')

const props = defineProps({
  year: {
    type: Number,
    default: () => dayjs().year()
  },
  month: {
    type: Number,
    default: () => dayjs().month() + 1
  }
})

const emit = defineEmits(['month-change'])

const currentYear = ref(props.year)
const currentMonth = ref(props.month)
const todos = ref({})
const showAddDialog = ref(false)
const showEditDialog = ref(false)
const selectedDate = ref('')
const selectedTodo = ref(null)
const newTodoText = ref('')

// 状态常量
const STATUS_OPTIONS = [
  { value: 'todo', label: '待办', color: 'bg-gray-100 text-gray-700 border-gray-300' },
  { value: 'in-progress', label: '进行中', color: 'bg-blue-100 text-blue-700 border-blue-300' },
  { value: 'completed', label: '已完成', color: 'bg-green-100 text-green-700 border-green-300' },
  { value: 'pending', label: '暂停', color: 'bg-yellow-100 text-yellow-700 border-yellow-300' },
  { value: 'cancelled', label: '已取消', color: 'bg-red-100 text-red-700 border-red-300' }
]

// 提醒类型
const REMINDER_TYPES = [
  { value: 'none', label: '不提醒' },
  { value: 'once', label: '一次性提醒' },
  { value: 'repeat', label: '重复提醒' }
]

// 重复规则类型
const REPEAT_RULES = [
  { value: 'day', label: '按天' },
  { value: 'weekday', label: '按星期' }
]

// 星期选项
const WEEKDAYS = [
  { value: 0, label: '周日' },
  { value: 1, label: '周一' },
  { value: 2, label: '周二' },
  { value: 3, label: '周三' },
  { value: 4, label: '周四' },
  { value: 5, label: '周五' },
  { value: 6, label: '周六' }
]

// 表单数据
const formStatus = ref('todo')
const formReminderEnabled = ref(false)
const formReminderType = ref('once')
const formReminderTime = ref('')
const formRepeatType = ref('day')
const formRepeatInterval = ref(1)
const formRepeatWeekdays = ref([1, 3, 5])

// 从 localStorage 加载待办事项
function loadTodos() {
  const saved = localStorage.getItem('calendar-todos')
  if (saved) {
    todos.value = JSON.parse(saved)
  }
}

// 保存待办事项到 localStorage
function saveTodos() {
  localStorage.setItem('calendar-todos', JSON.stringify(todos.value))
}

// 获取指定日期的待办事项
function getTodosForDate(dateStr) {
  return todos.value[dateStr] || []
}

// 获取状态样式类
function getStatusClass(status) {
  const statusMap = {
    'todo': 'bg-gray-100 text-gray-700 border border-gray-300',
    'in-progress': 'bg-blue-100 text-blue-700 border border-blue-300',
    'completed': 'bg-green-100 text-green-700 border border-green-300',
    'pending': 'bg-yellow-100 text-yellow-700 border border-yellow-300',
    'cancelled': 'bg-red-100 text-red-700 border border-red-300'
  }
  return statusMap[status] || 'bg-base-content/5 text-base-content/70 border border-base-content/10'
}

// 添加待办事项
function addTodo() {
  if (!newTodoText.value.trim()) return

  const dateStr = selectedDate.value
  if (!todos.value[dateStr]) {
    todos.value[dateStr] = []
  }

  const todo = {
    id: Date.now(),
    text: newTodoText.value,
    status: formStatus.value,
    reminder: formReminderEnabled.value ? {
      enabled: true,
      type: formReminderType.value,
      datetime: formReminderType.value === 'once' ? formReminderTime.value : undefined,
      repeatRule: formReminderType.value === 'repeat' ? {
        type: formRepeatType.value,
        value: formRepeatType.value === 'day' ? formRepeatInterval.value : formRepeatWeekdays.value
      } : undefined,
      nextReminder: calculateNextReminder({
        enabled: true,
        type: formReminderType.value,
        datetime: formReminderType.value === 'once' ? formReminderTime.value : undefined,
        repeatRule: formReminderType.value === 'repeat' ? {
          type: formRepeatType.value,
          value: formRepeatType.value === 'day' ? formRepeatInterval.value : formRepeatWeekdays.value
        } : undefined
      })
    } : undefined,
    createdAt: dayjs().toISOString(),
    updatedAt: dayjs().toISOString()
  }

  todos.value[dateStr].push(todo)
  saveTodos()
  resetForm()
}

// 计算下次提醒时间
function calculateNextReminder(reminder) {
  if (!reminder || !reminder.enabled) return undefined

  if (reminder.type === 'once') {
    return reminder.datetime
  }

  if (reminder.type === 'repeat' && reminder.repeatRule) {
    const now = dayjs()
    const rule = reminder.repeatRule

    if (rule.type === 'day') {
      return now.add(rule.value, 'day').set('hour', 9).set('minute', 0).toISOString()
    }

    if (rule.type === 'weekday') {
      const today = now.day()
      const targetDays = rule.value || []
      // 找下一个目标星期
      for (let i = 1; i <= 7; i++) {
        const nextDay = (today + i) % 7
        if (targetDays.includes(nextDay)) {
          return now.add(i, 'day').set('hour', 9).set('minute', 0).toISOString()
        }
      }
    }
  }

  return undefined
}

// 重置表单
function resetForm() {
  newTodoText.value = ''
  formStatus.value = 'todo'
  formReminderEnabled.value = false
  formReminderType.value = 'once'
  formReminderTime.value = ''
  formRepeatType.value = 'day'
  formRepeatInterval.value = 1
  formRepeatWeekdays.value = [1, 3, 5]
}

// 切换星期选择
function toggleWeekday(value) {
  const index = formRepeatWeekdays.value.indexOf(value)
  if (index > -1) {
    formRepeatWeekdays.value.splice(index, 1)
  } else {
    formRepeatWeekdays.value.push(value)
  }
}

// 打开编辑待办对话框
function openEditDialog(dateStr, todo) {
  selectedDate.value = dateStr
  selectedTodo.value = todo
  newTodoText.value = todo.text
  formStatus.value = todo.status || 'todo'

  if (todo.reminder && todo.reminder.enabled) {
    formReminderEnabled.value = true
    formReminderType.value = todo.reminder.type || 'once'
    formReminderTime.value = todo.reminder.datetime || ''
    if (todo.reminder.repeatRule) {
      formRepeatType.value = todo.reminder.repeatRule.type || 'day'
      formRepeatInterval.value = typeof todo.reminder.repeatRule.value === 'number' ? todo.reminder.repeatRule.value : 1
      formRepeatWeekdays.value = Array.isArray(todo.reminder.repeatRule.value) ? todo.reminder.repeatRule.value : [1, 3, 5]
    }
  } else {
    formReminderEnabled.value = false
  }

  showEditDialog.value = true
}

// 更新待办事项
function updateTodo() {
  if (!newTodoText.value.trim() || !selectedTodo.value) return

  const dateStr = selectedDate.value
  const todo = todos.value[dateStr]?.find(t => t.id === selectedTodo.value.id)
  if (todo) {
    todo.text = newTodoText.value
    todo.status = formStatus.value
    todo.updatedAt = dayjs().toISOString()

    if (formReminderEnabled.value) {
      todo.reminder = {
        enabled: true,
        type: formReminderType.value,
        datetime: formReminderType.value === 'once' ? formReminderTime.value : undefined,
        repeatRule: formReminderType.value === 'repeat' ? {
          type: formRepeatType.value,
          value: formRepeatType.value === 'day' ? formRepeatInterval.value : formRepeatWeekdays.value
        } : undefined,
        nextReminder: calculateNextReminder({
          enabled: true,
          type: formReminderType.value,
          datetime: formReminderType.value === 'once' ? formReminderTime.value : undefined,
          repeatRule: formReminderType.value === 'repeat' ? {
            type: formRepeatType.value,
            value: formRepeatType.value === 'day' ? formRepeatInterval.value : formRepeatWeekdays.value
          } : undefined
        })
      }
    } else {
      todo.reminder = undefined
    }

    saveTodos()
  }
  closeEditDialog()
}

// 删除待办事项
function deleteTodo() {
  if (!selectedTodo.value) return

  const dateStr = selectedDate.value
  if (todos.value[dateStr]) {
    todos.value[dateStr] = todos.value[dateStr].filter(t => t.id !== selectedTodo.value.id)
    if (todos.value[dateStr].length === 0) {
      delete todos.value[dateStr]
    }
    saveTodos()
  }
  closeEditDialog()
}

// 打开添加待办对话框
function openAddDialog(dateStr) {
  selectedDate.value = dateStr
  newTodoText.value = ''
  showAddDialog.value = true
}

// 关闭添加待办对话框
function closeAddDialog() {
  showAddDialog.value = false
  resetForm()
}

// 关闭编辑对话框
function closeEditDialog() {
  showEditDialog.value = false
  selectedTodo.value = null
  resetForm()
}

// 上个月
function prevMonth() {
  if (currentMonth.value === 1) {
    currentMonth.value = 12
    currentYear.value--
  } else {
    currentMonth.value--
  }
  emit('month-change', { year: currentYear.value, month: currentMonth.value })
}

// 下个月
function nextMonth() {
  if (currentMonth.value === 12) {
    currentMonth.value = 1
    currentYear.value++
  } else {
    currentMonth.value++
  }
  emit('month-change', { year: currentYear.value, month: currentMonth.value })
}

// 获取当前月份的天数
function getDaysInMonth() {
  return dayjs(`${currentYear.value}-${currentMonth.value}-1`).daysInMonth()
}

// 获取当前月份第一天是星期几（0-6，0是周日）
function getFirstDayOfWeek() {
  return dayjs(`${currentYear.value}-${currentMonth.value}-1`).day()
}

// 判断是否是今天
function isToday(day) {
  const today = dayjs()
  const date = dayjs(`${currentYear.value}-${currentMonth.value}-${day}`)
  return today.isSame(date, 'day')
}

// 判断是否是当前月份
function isCurrentMonth(dateStr) {
  const date = dayjs(dateStr)
  return date.year() === currentYear.value && date.month() + 1 === currentMonth.value
}

// 生成日历数据
const calendarDays = computed(() => {
  const days = []
  const daysInMonth = getDaysInMonth()
  const firstDayOfWeek = getFirstDayOfWeek()

  // 上个月的日期
  const prevMonthDays = dayjs(`${currentYear.value}-${currentMonth.value}-1`).subtract(1, 'month').daysInMonth()
  for (let i = firstDayOfWeek - 1; i >= 0; i--) {
    const day = prevMonthDays - i
    const dateStr = dayjs(`${currentYear.value}-${currentMonth.value}-1`)
      .subtract(firstDayOfWeek - i, 'day')
      .format('YYYY-MM-DD')
    days.push({
      day,
      dateStr,
      isCurrentMonth: false,
      isToday: isToday(day)
    })
  }

  // 当前月的日期
  for (let day = 1; day <= daysInMonth; day++) {
    const dateStr = `${currentYear.value}-${String(currentMonth.value).padStart(2, '0')}-${String(day).padStart(2, '0')}`
    days.push({
      day,
      dateStr,
      isCurrentMonth: true,
      isToday: isToday(day)
    })
  }

  // 下个月的日期
  const remainingDays = 42 - days.length
  for (let day = 1; day <= remainingDays; day++) {
    const dateStr = dayjs(`${currentYear.value}-${currentMonth.value}-1`)
      .add(daysInMonth + day - 1, 'day')
      .format('YYYY-MM-DD')
    days.push({
      day,
      dateStr,
      isCurrentMonth: false,
      isToday: isToday(day)
    })
  }

  return days
})

// 按周分组
const calendarWeeks = computed(() => {
  const weeks = []
  const days = calendarDays.value

  for (let i = 0; i < days.length; i += 7) {
    weeks.push(days.slice(i, i + 7))
  }

  return weeks
})

// 监听 props 变化
watch(() => props.year, (newVal) => {
  currentYear.value = newVal
})

watch(() => props.month, (newVal) => {
  currentMonth.value = newVal
})

onMounted(() => {
  loadTodos()
})
</script>

<template>
  <div class="calendar-todo h-full flex flex-col bg-base-100">
    <!-- 头部：月份导航 -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-base-200">
      <Button variant="ghost" size="sm" @click="prevMonth">
        <ChevronLeft :size="20" />
      </Button>
      <h2 class="text-lg font-semibold text-base-content">
        {{ currentYear }}年{{ currentMonth }}月
      </h2>
      <Button variant="ghost" size="sm" @click="nextMonth">
        <ChevronRight :size="20" />
      </Button>
    </div>

    <!-- 日历主体 -->
    <div class="flex-1 overflow-auto no-scrollbar relative flex flex-col">
      <!-- 星期标题 -->
      <div class="grid grid-cols-[repeat(7,minmax(100px,1fr))] border-b border-base-200 shrink-0">
        <div v-for="day in ['周日', '周一', '周二', '周三', '周四', '周五', '周六']" :key="day"
          class="py-2 text-center text-xs font-medium text-base-content/50 border-r border-b border-base-200 last:border-r-0">
          {{ day }}
        </div>
      </div>

      <!-- 所有日期格子放在一个统一的grid中 -->
      <div class="grid grid-cols-[repeat(7,minmax(100px,1fr))] grid-rows-[repeat(6,1fr)] min-h-0 flex-1">
        <div v-for="day in calendarDays" :key="day.dateStr"
          class="p-2 transition-colors hover:bg-base-content/5 cursor-pointer border-r border-b border-base-200 last:border-r-0"
          :class="{
            'bg-base-100': day.isCurrentMonth,
            'bg-base-content/5': !day.isCurrentMonth,
            'bg-primary/5': day.isToday
          }"
          @click.self="openAddDialog(day.dateStr)">
          <!-- 日期显示 -->
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-medium"
              :class="{
                'text-base-content/30': !day.isCurrentMonth,
                'text-primary': day.isToday,
                'text-base-content/70': day.isCurrentMonth && !day.isToday
              }">
              {{ day.day }}
            </span>
          </div>

          <!-- 待办标签列表 -->
          <div class="flex flex-col gap-1 overflow-hidden" @click.stop>
            <div v-for="todo in getTodosForDate(day.dateStr)" :key="todo.id"
              class="tag-todo px-2 py-0.5 rounded text-xs whitespace-nowrap overflow-hidden cursor-pointer transition-colors"
              :class="getStatusClass(todo.status)"
              @click="openEditDialog(day.dateStr, todo)">
              {{ todo.text }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加待办对话框 -->
    <div v-if="showAddDialog"
      class="fixed inset-0 bg-base-content/20 backdrop-blur-sm flex items-center justify-center z-50"
      @click.self="closeAddDialog">
      <div class="bg-base-100 rounded-lg shadow-xl p-6 w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto">
        <h3 class="text-lg font-semibold text-base-content mb-4">
          添加待办 - {{ selectedDate }}
        </h3>

        <!-- 内容输入 -->
        <textarea v-model="newTodoText" @keyup.enter.exact="addTodo" @keyup.enter.shift.exact.prevent
          placeholder="输入待办事项..."
          class="w-full min-h-[80px] px-3 py-2 border border-base-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent resize-none text-sm mb-4"></textarea>

        <!-- 状态选择 -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-base-content mb-2">状态</label>
          <div class="flex flex-wrap gap-2">
            <button v-for="status in STATUS_OPTIONS" :key="status.value" @click="formStatus = status.value"
              class="px-3 py-1.5 rounded text-xs border transition-colors"
              :class="formStatus === status.value ? status.color : 'bg-base-100 text-base-content/50 border-base-200'">
              {{ status.label }}
            </button>
          </div>
        </div>

        <!-- 提醒设置 -->
        <div class="mb-4">
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm font-medium text-base-content">提醒设置</label>
            <Toggle v-model="formReminderEnabled" size="sm" />
          </div>

          <div v-if="formReminderEnabled" class="space-y-3 ml-2 pl-4 border-l-2 border-base-200">
            <!-- 提醒类型 -->
            <div>
              <label class="block text-xs text-base-content/60 mb-1.5">提醒类型</label>
              <div class="flex gap-2">
                <button v-for="type in REMINDER_TYPES.slice(1)" :key="type.value" @click="formReminderType = type.value"
                  class="px-3 py-1.5 rounded text-xs border transition-colors"
                  :class="formReminderType === type.value ? 'bg-primary/10 text-primary border-primary/30' : 'bg-base-100 text-base-content/50 border-base-200'">
                  {{ type.label }}
                </button>
              </div>
            </div>

            <!-- 一次性提醒 -->
            <div v-if="formReminderType === 'once'">
              <label class="block text-xs text-base-content/60 mb-1.5">提醒时间</label>
              <input type="datetime-local" v-model="formReminderTime"
                class="w-full px-3 py-1.5 border border-base-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/50">
            </div>

            <!-- 重复提醒 -->
            <div v-if="formReminderType === 'repeat'">
              <label class="block text-xs text-base-content/60 mb-1.5">重复规则</label>
              <div class="flex gap-2 mb-2">
                <button v-for="rule in REPEAT_RULES" :key="rule.value" @click="formRepeatType = rule.value"
                  class="px-3 py-1.5 rounded text-xs border transition-colors"
                  :class="formRepeatType === rule.value ? 'bg-primary/10 text-primary border-primary/30' : 'bg-base-100 text-base-content/50 border-base-200'">
                  {{ rule.label }}
                </button>
              </div>

              <!-- 按天重复 -->
              <div v-if="formRepeatType === 'day'" class="flex items-center gap-2">
                <span class="text-xs text-base-content/60">每隔</span>
                <input type="number" v-model="formRepeatInterval" min="1" max="365"
                  class="w-16 px-2 py-1 border border-base-200 rounded text-sm text-center">
                <span class="text-xs text-base-content/60">天</span>
              </div>

              <!-- 按星期重复 -->
              <div v-if="formRepeatType === 'weekday'">
                <div class="flex flex-wrap gap-1.5">
                  <button v-for="wd in WEEKDAYS" :key="wd.value" @click="toggleWeekday(wd.value)"
                    class="w-8 h-8 rounded text-xs border transition-colors"
                    :class="formRepeatWeekdays.includes(wd.value) ? 'bg-primary text-primary-content border-primary' : 'bg-base-100 text-base-content/50 border-base-200'">
                    {{ wd.label }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
          <Button variant="secondary" size="sm" @click="closeAddDialog">
            取消
          </Button>
          <Button variant="primary" size="sm" @click="addTodo">
            添加
          </Button>
        </div>
      </div>
    </div>

    <!-- 编辑待办对话框 -->
    <div v-if="showEditDialog"
      class="fixed inset-0 bg-base-content/20 backdrop-blur-sm flex items-center justify-center z-50"
      @click.self="closeEditDialog">
      <div class="bg-base-100 rounded-lg shadow-xl p-6 w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto">
        <h3 class="text-lg font-semibold text-base-content mb-4">
          编辑待办 - {{ selectedDate }}
        </h3>

        <!-- 内容输入 -->
        <textarea v-model="newTodoText" @keyup.enter.exact="updateTodo" @keyup.enter.shift.exact.prevent
          placeholder="输入待办事项..."
          class="w-full min-h-[80px] px-3 py-2 border border-base-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent resize-none text-sm mb-4"></textarea>

        <!-- 状态选择 -->
        <div class="mb-4">
          <label class="block text-sm font-medium text-base-content mb-2">状态</label>
          <div class="flex flex-wrap gap-2">
            <button v-for="status in STATUS_OPTIONS" :key="status.value" @click="formStatus = status.value"
              class="px-3 py-1.5 rounded text-xs border transition-colors"
              :class="formStatus === status.value ? status.color : 'bg-base-100 text-base-content/50 border-base-200'">
              {{ status.label }}
            </button>
          </div>
        </div>

        <!-- 提醒设置 -->
        <div class="mb-4">
          <div class="flex items-center justify-between mb-2">
            <label class="text-sm font-medium text-base-content">提醒设置</label>
            <Toggle v-model="formReminderEnabled" size="sm" />
          </div>

          <div v-if="formReminderEnabled" class="space-y-3 ml-2 pl-4 border-l-2 border-base-200">
            <!-- 提醒类型 -->
            <div>
              <label class="block text-xs text-base-content/60 mb-1.5">提醒类型</label>
              <div class="flex gap-2">
                <button v-for="type in REMINDER_TYPES.slice(1)" :key="type.value" @click="formReminderType = type.value"
                  class="px-3 py-1.5 rounded text-xs border transition-colors"
                  :class="formReminderType === type.value ? 'bg-primary/10 text-primary border-primary/30' : 'bg-base-100 text-base-content/50 border-base-200'">
                  {{ type.label }}
                </button>
              </div>
            </div>

            <!-- 一次性提醒 -->
            <div v-if="formReminderType === 'once'">
              <label class="block text-xs text-base-content/60 mb-1.5">提醒时间</label>
              <input type="datetime-local" v-model="formReminderTime"
                class="w-full px-3 py-1.5 border border-base-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary/50">
            </div>

            <!-- 重复提醒 -->
            <div v-if="formReminderType === 'repeat'">
              <label class="block text-xs text-base-content/60 mb-1.5">重复规则</label>
              <div class="flex gap-2 mb-2">
                <button v-for="rule in REPEAT_RULES" :key="rule.value" @click="formRepeatType = rule.value"
                  class="px-3 py-1.5 rounded text-xs border transition-colors"
                  :class="formRepeatType === rule.value ? 'bg-primary/10 text-primary border-primary/30' : 'bg-base-100 text-base-content/50 border-base-200'">
                  {{ rule.label }}
                </button>
              </div>

              <!-- 按天重复 -->
              <div v-if="formRepeatType === 'day'" class="flex items-center gap-2">
                <span class="text-xs text-base-content/60">每隔</span>
                <input type="number" v-model="formRepeatInterval" min="1" max="365"
                  class="w-16 px-2 py-1 border border-base-200 rounded text-sm text-center">
                <span class="text-xs text-base-content/60">天</span>
              </div>

              <!-- 按星期重复 -->
              <div v-if="formRepeatType === 'weekday'">
                <div class="flex flex-wrap gap-1.5">
                  <button v-for="wd in WEEKDAYS" :key="wd.value" @click="toggleWeekday(wd.value)"
                    class="w-8 h-8 rounded text-xs border transition-colors"
                    :class="formRepeatWeekdays.includes(wd.value) ? 'bg-primary text-primary-content border-primary' : 'bg-base-100 text-base-content/50 border-base-200'">
                    {{ wd.label }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="flex justify-between items-center mt-6">
          <Button variant="ghost" size="sm" @click="deleteTodo" class="text-error hover:text-error hover:bg-error/10">
            删除
          </Button>
          <div class="flex gap-2">
            <Button variant="secondary" size="sm" @click="closeEditDialog">
              取消
            </Button>
            <Button variant="primary" size="sm" @click="updateTodo">
              保存
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped></style>
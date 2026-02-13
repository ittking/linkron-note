<script setup>
import { ref, computed } from 'vue'
import { ChevronLeft, ChevronRight, X } from 'lucide-vue-next'
import dayjs from 'dayjs'
import Button from './ui/Button.vue'
import Toggle from './ui/Toggle.vue'
import Select from './ui/Select.vue'
import DateTimePicker from './ui/DateTimePicker.vue'

dayjs.locale('zh-cn')

const props = defineProps({
  year: {
    type: Number,
    default: () => dayjs().year()
  },
  month: {
    type: Number,
    default: () => dayjs().month() + 1
  },
  todos: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['month-change', 'create', 'update', 'delete', 'open-edit'])

const currentYear = ref(props.year)
const currentMonth = ref(props.month)

// 获取指定日期的待办事项
function getTodosForDate(dateStr) {
  return props.todos.filter(t => t.date === dateStr) || []
}

// 获取状态样式类
function getStatusClass(status) {
  const statusMap = {
    'todo': 'bg-gray-100 text-gray-700 border border-gray-300',
    'in-progress': 'bg-blue-100 text-blue-700 border border-blue-300',
    'completed': 'bg-green-100 text-green-700 border border-green-300 line-through opacity-70',
    'pending': 'bg-yellow-100 text-yellow-700 border border-yellow-300 line-through opacity-70',
    'cancelled': 'bg-red-100 text-red-700 border border-red-300 line-through opacity-70'
  }
  return statusMap[status] || 'bg-base-content/5 text-base-content/70 border border-base-content/10'
}

// 打开待办对话框
function openTodoDialog(dateStr, todo = null) {
  emit('open-edit', dateStr, todo)
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
      <div class="grid grid-cols-[repeat(7,minmax(100px,1fr))] shrink-0">
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
          @click="openTodoDialog(day.dateStr)">
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
          <div class="flex flex-col gap-1 overflow-hidden">
            <div v-for="todo in getTodosForDate(day.dateStr)" :key="todo.id"
              class="tag-todo px-2 py-0.5 rounded text-xs cursor-pointer transition-colors"
              :class="getStatusClass(todo.status)"
              @click.stop="openTodoDialog(day.dateStr, todo)">
              <div class="overflow-hidden text-ellipsis whitespace-nowrap">
                {{ todo.text }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
      </div>
    </template>

<style scoped></style>
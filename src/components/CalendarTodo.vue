<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { ChevronLeft, ChevronRight, Plus, X, Check } from 'lucide-vue-next'
import dayjs from 'dayjs'

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
const selectedDate = ref('')
const newTodoText = ref('')

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

// 添加待办事项
function addTodo() {
  if (!newTodoText.value.trim()) return
  
  const dateStr = selectedDate.value
  if (!todos.value[dateStr]) {
    todos.value[dateStr] = []
  }
  
  todos.value[dateStr].push({
    id: Date.now(),
    text: newTodoText.value,
    completed: false,
    createdAt: dayjs().toISOString()
  })
  
  saveTodos()
  newTodoText.value = ''
}

// 切换待办完成状态
function toggleTodo(dateStr, todoId) {
  const todo = todos.value[dateStr]?.find(t => t.id === todoId)
  if (todo) {
    todo.completed = !todo.completed
    saveTodos()
  }
}

// 删除待办事项
function deleteTodo(dateStr, todoId) {
  if (todos.value[dateStr]) {
    todos.value[dateStr] = todos.value[dateStr].filter(t => t.id !== todoId)
    if (todos.value[dateStr].length === 0) {
      delete todos.value[dateStr]
    }
    saveTodos()
  }
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
  newTodoText.value = ''
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
      <button @click="prevMonth" class="p-2 rounded hover:bg-base-200 transition-colors">
        <ChevronLeft :size="20" class="text-base-content/70" />
      </button>
      <h2 class="text-lg font-semibold text-base-content">
        {{ currentYear }}年{{ currentMonth }}月
      </h2>
      <button @click="nextMonth" class="p-2 rounded hover:bg-base-200 transition-colors">
        <ChevronRight :size="20" class="text-base-content/70" />
      </button>
    </div>

    <!-- 星期标题 -->
    <div class="grid grid-cols-7 gap-px bg-base-200 border-b border-base-200">
      <div v-for="day in ['周日', '周一', '周二', '周三', '周四', '周五', '周六']" :key="day"
        class="bg-base-100 py-2 text-center text-xs font-medium text-base-content/50">
        {{ day }}
      </div>
    </div>

    <!-- 日历主体 -->
    <div class="flex-1 overflow-y-auto  no-scrollbar">
      <div v-for="(week, weekIndex) in calendarWeeks" :key="weekIndex"
        class="grid grid-cols-7 gap-px bg-base-200 border-b border-base-200 last:border-b-0">
        <div v-for="day in week" :key="day.dateStr"
          class="min-h-[120px] bg-base-100 p-2 transition-colors hover:bg-base-content/5 cursor-pointer"
          :class="{
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

          <!-- 待办列表 -->
          <div class="space-y-1" @click.stop>
            <div v-for="todo in getTodosForDate(day.dateStr).slice(0, 3)" :key="todo.id"
              class="flex items-start gap-1 group text-xs">
              <button @click="toggleTodo(day.dateStr, todo.id)"
                class="flex-shrink-0 mt-0.5 w-3 h-3 rounded border transition-colors"
                :class="{
                  'border-primary bg-primary': todo.completed,
                  'border-base-content/30': !todo.completed
                }">
                <Check v-if="todo.completed" :size="10" class="text-white" />
              </button>
              <span class="flex-1 truncate leading-tight"
                :class="{
                  'line-through text-base-content/30': todo.completed,
                  'text-base-content/70': !todo.completed
                }">
                {{ todo.text }}
              </span>
              <button @click="deleteTodo(day.dateStr, todo.id)"
                class="flex-shrink-0 opacity-0 group-hover:opacity-100 p-0.5 hover:text-error transition-all">
                <X :size="10" />
              </button>
            </div>
            <div v-if="getTodosForDate(day.dateStr).length > 3"
              class="text-xs text-base-content/30 pl-4" @click.stop>
              还有 {{ getTodosForDate(day.dateStr).length - 3 }} 项
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 添加待办对话框 -->
    <div v-if="showAddDialog" class="fixed inset-0 bg-base-content/20 backdrop-blur-sm flex items-center justify-center z-50"
      @click.self="closeAddDialog">
      <div class="bg-base-100 rounded-lg shadow-xl p-6 w-full max-w-md mx-4">
        <h3 class="text-lg font-semibold text-base-content mb-4">
          添加待办 - {{ selectedDate }}
        </h3>
        <textarea
          v-model="newTodoText"
          @keyup.enter.exact="addTodo"
          @keyup.enter.shift.exact.prevent
          placeholder="输入待办事项..."
          class="w-full min-h-[80px] px-3 py-2 border border-base-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent resize-none text-sm"
          ref="textareaRef"
        ></textarea>
        <div class="flex justify-end gap-2 mt-4">
          <button @click="closeAddDialog"
            class="px-4 py-2 text-sm text-base-content/70 hover:bg-base-200 rounded-lg transition-colors">
            取消
          </button>
          <button @click="addTodo"
            class="px-4 py-2 text-sm bg-primary text-primary-content rounded-lg hover:bg-primary/90 transition-colors">
            添加
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>

</style>
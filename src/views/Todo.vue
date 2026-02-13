<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import CalendarTodo from '@/components/CalendarTodo.vue'
import TodayTodoList from '@/components/TodayTodoList.vue'
import { Calendar, CheckSquare } from 'lucide-vue-next'
import dayjs from 'dayjs'
import { useWorkDirectory } from '@/composables/useWorkDirectory'

dayjs.locale('zh-cn')

const { getWorkDirectory } = useWorkDirectory('setting')

// 当前视图：'calendar' 或 'today'
const currentView = ref('today')

// 日历视图的年月
const calendarYear = ref(dayjs().year())
const calendarMonth = ref(dayjs().month() + 1)

// 数据状态
const todayTodos = ref([])
const monthTodos = ref([])
const loading = ref(false)

// 状态颜色映射
const STATUS_COLORS = {
  'todo': '#6B7280',
  'in-progress': '#3B82F6',
  'completed': '#10B981',
  'pending': '#F59E0B',
  'cancelled': '#EF4444'
}

// 获取今日日期字符串
const today = computed(() => dayjs().format('YYYY-MM-DD'))

// 今日显示的日期信息
const todayDisplay = computed(() => {
  return {
    full: `今日 · ${dayjs().format('YYYY年M月D日')} · ${dayjs().format('dddd')}`
  }
})

// 获取状态颜色
function getStatusColor(status) {
  return STATUS_COLORS[status] || STATUS_COLORS['todo']
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

// 从后端加载今日待办事项
async function loadTodayTodos() {
  loading.value = true
  try {
    const workDirectory = await getWorkDirectory()
    const data = await invoke('get_today_todos', { 
      todayDate: today.value,
      workDirectory 
    })
    todayTodos.value = data
  } catch (error) {
    console.error('加载今日待办事项失败:', error)
  } finally {
    loading.value = false
  }
}

// 从后端加载月度待办事项
async function loadMonthTodos() {
  loading.value = true
  try {
    const workDirectory = await getWorkDirectory()
    const data = await invoke('get_todos_by_month', { 
      year: calendarYear.value, 
      month: calendarMonth.value,
      workDirectory 
    })
    monthTodos.value = data
  } catch (error) {
    console.error('加载月度待办事项失败:', error)
  } finally {
    loading.value = false
  }
}

// 创建待办事项
async function createTodo(data) {
  try {
    const workDirectory = await getWorkDirectory()
    
    await invoke('create_todo', {
      date: data.date,
      text: data.text,
      status: data.status || 'todo',
      reminder: data.reminder || null,
      workDirectory
    })

    // 重新加载数据
    await loadTodayTodos()
    await loadMonthTodos()
  } catch (error) {
    console.error('创建待办失败:', error)
    throw error
  }
}

// 更新待办事项
async function updateTodo(data) {
  try {
    const workDirectory = await getWorkDirectory()
    
    await invoke('update_todo', {
      id: data.id,
      text: data.text,
      status: data.status,
      reminder: data.reminder || null,
      workDirectory
    })

    // 重新加载数据
    await loadTodayTodos()
    await loadMonthTodos()
  } catch (error) {
    console.error('更新待办失败:', error)
    throw error
  }
}

// 删除待办事项
async function deleteTodo(id) {
  try {
    const workDirectory = await getWorkDirectory()
    await invoke('delete_todo', {
      id,
      workDirectory
    })

    // 重新加载数据
    await loadTodayTodos()
    await loadMonthTodos()
  } catch (error) {
    console.error('删除待办失败:', error)
    throw error
  }
}

// 切换待办完成状态
async function toggleTodoStatus(todo) {
  const newStatus = todo.status === 'completed' ? 'todo' : 'completed'
  await updateTodo({
    id: todo.id,
    text: todo.text,
    status: newStatus,
    reminder: todo.reminder ? JSON.stringify(todo.reminder) : null
  })
}

// 切换视图
function toggleView() {
  currentView.value = currentView.value === 'calendar' ? 'today' : 'calendar'
}

// 处理月份变化
function handleMonthChange(data) {
  calendarYear.value = data.year
  calendarMonth.value = data.month
}

// 监听月份变化，重新加载月度数据
watch([calendarYear, calendarMonth], () => {
  loadMonthTodos()
})

onMounted(() => {
  loadTodayTodos()
  loadMonthTodos()
})
</script>

<template>
  <div class="h-full relative">
    <!-- 日历视图 -->
    <CalendarTodo 
      v-if="currentView === 'calendar'"
      :year="calendarYear"
      :month="calendarMonth"
      :todos="monthTodos"
      :loading="loading"
      @month-change="handleMonthChange"
      @create="createTodo"
      @update="updateTodo"
      @delete="deleteTodo"
    />
    
    <!-- 今日列表视图 -->
    <TodayTodoList 
      v-else
      :todos="todayTodos"
      :loading="loading"
      @create="createTodo"
      @update="updateTodo"
      @delete="deleteTodo"
      @toggle-status="toggleTodoStatus"
    />

    <!-- 悬浮切换按钮 -->
    <button
      @click="toggleView"
      class="fixed bottom-6 right-6 z-50 w-10 h-10 bg-primary text-primary-content rounded-full flex items-center justify-center shadow-lg hover:bg-primary/90 hover:scale-105 transition-all duration-200"
      :title="currentView === 'calendar' ? '切换到今日列表' : '切换到日历视图'"
    >
      <Calendar v-if="currentView === 'today'" :size="18" />
      <CheckSquare v-else :size="18" />
    </button>
  </div>
</template>
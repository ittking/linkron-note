<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import CalendarTodo from '@/components/CalendarTodo.vue'
import TodayTodoList from '@/components/TodayTodoList.vue'
import TodoDialog from '@/components/TodoDialog.vue'
import { Calendar, CheckSquare } from 'lucide-vue-next'
import dayjs from 'dayjs'
import { useWorkDirectory } from '@/composables/useWorkDirectory'
import { useSync } from '@/composables/useSync'
import { useConfirmDialog } from '@/composables/useConfirmDialog'

dayjs.locale('zh-cn')

const { getWorkDirectory } = useWorkDirectory('setting')
const { triggerSync } = useSync()

// 当前视图：'calendar' 或 'today'
const currentView = ref('today')

// 日历视图的年月
const calendarYear = ref(dayjs().year())
const calendarMonth = ref(dayjs().month() + 1)

// 数据状态
const todayTodos = ref([])
const monthTodos = ref([])
const loading = ref(false)

// 获取今日日期字符串
const today = computed(() => dayjs().format('YYYY-MM-DD'))

// 编辑对话框状态
const showEditDialog = ref(false)
const editDialogDate = ref('')
const editDialogTodo = ref(null)

// 删除确认对话框
const { confirmVisible, confirmTitle, confirmContent, showConfirm, handleConfirmOk } = useConfirmDialog()
const pendingDeleteId = ref(null)

// 打开编辑对话框
function openEditDialog(date, todo = null) {
  editDialogDate.value = date
  editDialogTodo.value = todo
  showEditDialog.value = true
}

// 处理编辑对话框保存
function handleDialogSave(data) {
  if (data.id) {
    updateTodo(data)
  } else {
    createTodo(data)
  }
}

// 处理编辑对话框删除
function handleDialogDelete(id) {
  pendingDeleteId.value = id
  showConfirm('确认删除', '确定要删除这个待办事项吗？删除后无法恢复。', confirmDeleteTodo)
}

// 辅助函数：解析 reminder JSON
function parseReminder(reminderStr) {
  if (!reminderStr) return null
  try {
    return JSON.parse(reminderStr)
  } catch {
    return null
  }
}

function stringifyReminder(reminderObj) {
  if (!reminderObj) return null
  try {
    return JSON.stringify(reminderObj)
  } catch {
    return null
  }
}

function createTodoItem(data, id) {
  return {
    id: id,
    date: data.date,
    text: data.text,
    status: data.status || 'todo',
    reminder: parseReminder(data.reminder),
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString()
  }
}

function updateTodoItemInArray(items, data) {
  const index = items.findIndex(t => t.id === data.id)
  if (index !== -1) {
    items[index] = {
      ...items[index],
      text: data.text,
      status: data.status,
      reminder: parseReminder(data.reminder),
      date: data.date || items[index].date
    }
  }
}

// 本地排序函数：未完成 > 进行中 > 暂停 > 已取消 > 已完成；同状态按创建时间排序（新的在前）
function sortTodosLocally(todos) {
  return [...todos].sort((a, b) => {
    // 定义状态优先级：数字越小优先级越高
    const statusPriority = (status) => {
      switch (status) {
        case 'todo': return 1           // 未完成 - 最高优先级
        case 'in-progress': return 2    // 进行中
        case 'pending': return 3        // 暂停
        case 'cancelled': return 4      // 已取消
        case 'completed': return 5      // 已完成 - 最低优先级
        default: return 6               // 其他状态
      }
    }

    const aPriority = statusPriority(a.status)
    const bPriority = statusPriority(b.status)

    if (aPriority !== bPriority) {
      // 优先级不同，按优先级排序（数字小的在前）
      return aPriority - bPriority
    }

    // 同状态按创建时间排序（新的在前）
    const aTime = dayjs(a.created_at).valueOf()
    const bTime = dayjs(b.created_at).valueOf()
    return bTime - aTime
  })
}

async function loadTodayTodos(date = today.value) {
  loading.value = true
  try {
    const workDirectory = await getWorkDirectory()
    const data = await invoke('get_todos_by_date', { 
      date,
      workDirectory 
    })
    todayTodos.value = sortTodosLocally(data)
  } catch (error) {
    console.error('加载待办事项失败:', error)
  } finally {
    loading.value = false
  }
}

function handleDateChange(date) {
  loadTodayTodos(date)
}

async function loadMonthTodos() {
  loading.value = true
  try {
    const workDirectory = await getWorkDirectory()
    const data = await invoke('get_todos_by_month', { 
      year: calendarYear.value, 
      month: calendarMonth.value,
      workDirectory 
    })
    monthTodos.value = sortTodosLocally(data)
  } catch (error) {
    console.error('加载月度待办事项失败:', error)
  } finally {
    loading.value = false
  }
}

async function createTodo(data) {
  try {
    const workDirectory = await getWorkDirectory()

    const result = await invoke('create_todo', {
      date: data.date,
      text: data.text,
      status: data.status || 'todo',
      reminder: data.reminder || null,
      workDirectory
    })

    const newTodo = createTodoItem(data, result)

    todayTodos.value.push(newTodo)
    monthTodos.value.push(newTodo)
    todayTodos.value = sortTodosLocally(todayTodos.value)

    // 触发自动同步
    triggerSync()
  } catch (error) {
    console.error('创建待办失败:', error)
    throw error
  }
}

async function updateTodo(data) {
  try {
    const workDirectory = await getWorkDirectory()

    await invoke('update_todo', {
      id: data.id,
      text: data.text,
      status: data.status,
      reminder: data.reminder || null,
      date: data.date || null,
      workDirectory
    })

    updateTodoItemInArray(todayTodos.value, data)
    updateTodoItemInArray(monthTodos.value, data)
    todayTodos.value = sortTodosLocally(todayTodos.value)

    // 触发自动同步
    triggerSync()
  } catch (error) {
    console.error('更新待办失败:', error)
    throw error
  }
}

// 点击删除按钮，显示确认对话框
function requestDeleteTodo(id) {
  pendingDeleteId.value = id
  showConfirm('确认删除', '确定要删除这个待办事项吗？删除后无法恢复。', confirmDeleteTodo)
}

// 确认删除
async function confirmDeleteTodo() {
  const id = pendingDeleteId.value
  if (!id) return
  
  try {
    const workDirectory = await getWorkDirectory()
    await invoke('delete_todo', {
      id,
      workDirectory
    })

    todayTodos.value = todayTodos.value.filter(t => t.id !== id)
    monthTodos.value = monthTodos.value.filter(t => t.id !== id)

    // 触发自动同步
    triggerSync()
  } catch (error) {
    console.error('删除待办失败:', error)
    throw error
  }
}

// 切换待办完成状态
async function toggleTodoStatus(todo) {
  // 如果是已完成、暂停或已取消，变成待办；否则变成已完成
  const newStatus = ['completed', 'pending', 'cancelled'].includes(todo.status) 
    ? 'todo' 
    : 'completed'
  await updateTodo({
    id: todo.id,
    text: todo.text,
    status: newStatus,
    reminder: stringifyReminder(todo.reminder)
  })
}

function toggleView() {
  if (currentView.value === 'calendar') {
    currentView.value = 'today'
    loadTodayTodos()
  } else {
    currentView.value = 'calendar'
    loadMonthTodos()
  }
}

function handleMonthChange(data) {
  calendarYear.value = data.year
  calendarMonth.value = data.month
}

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
       @delete="requestDeleteTodo"
       @open-edit="openEditDialog"
     />
    
     <!-- 今日列表视图 -->
     <TodayTodoList 
       v-else
       :todos="todayTodos"
       :loading="loading"
       @create="createTodo"
       @update="updateTodo"
       @delete="requestDeleteTodo"
       @toggle-status="toggleTodoStatus"
       @date-change="handleDateChange"
       @open-edit="openEditDialog"
     />

     <!-- 编辑对话框 -->
     <TodoDialog
       v-model:show="showEditDialog"
       :date="editDialogDate"
       :todo="editDialogTodo"
       @save="handleDialogSave"
       @delete="handleDialogDelete"
     />

      <!-- 删除确认对话框 -->
      <div v-if="confirmVisible" class="modal modal-open">
        <div class="modal-box">
          <h3 class="font-bold text-lg">{{ confirmTitle }}</h3>
          <p class="py-4 text-base-content/70">{{ confirmContent }}</p>
          <div class="modal-action justify-end gap-2">
            <button class="btn" @click="confirmVisible = false">取消</button>
            <button class="btn btn-error" @click="handleConfirmOk">确定删除</button>
          </div>
        </div>
        <div class="modal-backdrop" @click="confirmVisible = false"></div>
      </div>

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
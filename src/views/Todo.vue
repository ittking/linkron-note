<script setup>
import { ref } from 'vue'
import CalendarTodo from '@/components/CalendarTodo.vue'
import TodayTodoList from '@/components/TodayTodoList.vue'
import { Calendar, CheckSquare } from 'lucide-vue-next'

// 当前视图：'calendar' 或 'today'
const currentView = ref('today')

// 切换视图
function toggleView() {
  currentView.value = currentView.value === 'calendar' ? 'today' : 'calendar'
}

// 处理月份变化
function handleMonthChange(data) {
  console.log('月份变化:', data)
}
</script>

<template>
  <div class="h-full relative">
    <!-- 日历视图 -->
    <CalendarTodo v-if="currentView === 'calendar'" @month-change="handleMonthChange" />
    
    <!-- 今日列表视图 -->
    <TodayTodoList v-else />

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
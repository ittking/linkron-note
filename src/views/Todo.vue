<script setup>
import CalendarTodo from '@/components/CalendarTodo.vue'

function handleMonthChange(data) {
  console.log('月份变化:', data)
}
</script>

<template>
  <div class="h-full">
    <CalendarTodo @month-change="handleMonthChange" />
  </div>
</template>
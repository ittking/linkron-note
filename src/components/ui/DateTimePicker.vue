<script setup>
import { ref, computed, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight, Clock, X } from 'lucide-vue-next'
import dayjs from 'dayjs'
import dayjsLocale from 'dayjs/locale/zh-cn'

dayjs.locale(dayjsLocale)

const props = defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  placeholder: {
    type: String,
    default: '选择日期时间'
  },
  mode: {
    type: String,
    default: 'datetime',
    validator: (value) => ['date', 'time', 'datetime'].includes(value)
  },
  disabled: {
    type: Boolean,
    default: false
  },
  min: {
    type: String,
    default: ''
  },
  clearable: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:model-value', 'update:modelValue', 'change'])

const isOpen = ref(false)
const dropdownRef = ref(null)
const triggerRef = ref(null)
const calendarRef = ref(null)
const hourScrollRef = ref(null)
const minuteScrollRef = ref(null)
const timePickerHeight = ref('200px')

// 临时值（用于确认前暂存）
const tempSelectedDate = ref(dayjs().format('YYYY-MM-DD'))
const tempSelectedHour = ref('')
const tempSelectedMinute = ref('')

// 已确认的值
const selectedDate = ref(dayjs().format('YYYY-MM-DD'))
const selectedHour = ref('')
const selectedMinute = ref('')

// 日历状态
const currentYear = ref(dayjs().year())
const currentMonth = ref(dayjs().month() + 1)
const showYearPicker = ref(false)

// 年份选择范围
const yearRange = computed(() => {
  const current = dayjs().year()
  const startYear = current - 10
  const endYear = current + 10
  const years = []
  for (let y = startYear; y <= endYear; y++) {
    years.push(y)
  }
  return years
})

// 年份分页
const yearPageStart = ref(0)
const displayedYears = computed(() => {
  return yearRange.value.slice(yearPageStart.value, yearPageStart.value + 12)
})

// 下拉框定位
const dropdownPosition = ref({
  top: '100%',
  bottom: 'auto',
  left: '0',
  right: 'auto'
})

// 初始化值
const initValue = () => {
  if (props.modelValue) {
    let date
    if (props.mode === 'time') {
      // time 模式下，modelValue 可能是 HH:mm 或 YYYY-MM-DDTHH:mm
      if (props.modelValue.includes('T')) {
        date = dayjs(props.modelValue)
      } else {
        // 只有时间部分，用今天的日期补全
        date = dayjs(`${dayjs().format('YYYY-MM-DD')}T${props.modelValue}`)
      }
    } else {
      date = dayjs(props.modelValue)
    }

    if (date.isValid()) {
      selectedDate.value = date.format('YYYY-MM-DD')
      selectedHour.value = date.format('HH')
      selectedMinute.value = date.format('mm')
    } else {
      // 无效日期，使用默认值（当前时间）
      selectedDate.value = dayjs().format('YYYY-MM-DD')
      selectedHour.value = dayjs().format('HH')
      selectedMinute.value = dayjs().format('mm')
    }
  } else {
    // 使用当前时间作为默认值
    const now = dayjs()
    selectedDate.value = now.format('YYYY-MM-DD')
    selectedHour.value = now.format('HH')
    selectedMinute.value = now.format('mm')
  }
  tempSelectedDate.value = selectedDate.value
  tempSelectedHour.value = selectedHour.value
  tempSelectedMinute.value = selectedMinute.value
}

// 格式化显示值
const displayValue = computed(() => {
  if (!props.modelValue) return props.placeholder

  let date
  if (props.mode === 'time') {
    // time 模式下，modelValue 可能是 HH:mm 或 YYYY-MM-DDTHH:mm
    if (props.modelValue.includes('T')) {
      date = dayjs(props.modelValue)
    } else {
      // 只有时间部分，用今天的日期补全
      date = dayjs(`${dayjs().format('YYYY-MM-DD')}T${props.modelValue}`)
    }
  } else {
    date = dayjs(props.modelValue)
  }

  if (!date.isValid()) return props.placeholder

  const dateStr = date.format('YYYY/MM/DD')
  const timeStr = date.format('HH:mm')

  if (props.mode === 'date') return dateStr
  if (props.mode === 'time') return timeStr
  return `${dateStr} ${timeStr}`
})

const hasValue = computed(() => !!props.modelValue)

// 获取当前月份的天数
function getDaysInMonth() {
  return dayjs(`${currentYear.value}-${currentMonth.value}-1`).daysInMonth()
}

// 获取当前月份第一天是星期几
function getFirstDayOfWeek() {
  return dayjs(`${currentYear.value}-${currentMonth.value}-1`).day()
}

// 生成日历数据
const calendarDays = computed(() => {
  const days = []
  const daysInMonth = getDaysInMonth()
  const firstDayOfWeek = getFirstDayOfWeek()

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
      isToday: dayjs(dateStr).isSame(dayjs(), 'day'),
      isSelected: dateStr === tempSelectedDate.value
    })
  }

  for (let day = 1; day <= daysInMonth; day++) {
    const dateStr = `${currentYear.value}-${String(currentMonth.value).padStart(2, '0')}-${String(day).padStart(2, '0')}`
    days.push({
      day,
      dateStr,
      isCurrentMonth: true,
      isToday: dayjs(dateStr).isSame(dayjs(), 'day'),
      isSelected: dateStr === tempSelectedDate.value
    })
  }

  const remainingDays = 42 - days.length
  for (let day = 1; day <= remainingDays; day++) {
    const dateStr = dayjs(`${currentYear.value}-${currentMonth.value}-1`)
      .add(daysInMonth + day - 1, 'day')
      .format('YYYY-MM-DD')
    days.push({
      day,
      dateStr,
      isCurrentMonth: false,
      isToday: dayjs(dateStr).isSame(dayjs(), 'day'),
      isSelected: dateStr === tempSelectedDate.value
    })
  }

  return days
})

const calendarWeeks = computed(() => {
  const weeks = []
  const days = calendarDays.value
  for (let i = 0; i < days.length; i += 7) {
    weeks.push(days.slice(i, i + 7))
  }
  return weeks
})

function prevYear() {
  currentYear.value--
  calculateTimePickerHeight()
}

function nextYear() {
  currentYear.value++
  calculateTimePickerHeight()
}

function prevMonth() {
  if (currentMonth.value === 1) {
    currentMonth.value = 12
    currentYear.value--
  } else {
    currentMonth.value--
  }
  calculateTimePickerHeight()
}

function nextMonth() {
  if (currentMonth.value === 12) {
    currentMonth.value = 1
    currentYear.value++
  } else {
    currentMonth.value++
  }
  calculateTimePickerHeight()
}

function toggleYearPicker() {
  showYearPicker.value = !showYearPicker.value
  nextTick(() => {
    calculateTimePickerHeight()
  })
}

function selectYear(year) {
  currentYear.value = year
  showYearPicker.value = false
  calculatePosition()
}

function prevYearPage() {
  yearPageStart.value = Math.max(0, yearPageStart.value - 12)
}

function nextYearPage() {
  yearPageStart.value = Math.min(yearRange.value.length - 12, yearPageStart.value + 12)
}

function selectDate(dateStr) {
  tempSelectedDate.value = dateStr
}

const hourOptions = computed(() => {
  return Array.from({ length: 24 }, (_, i) => ({
    label: String(i).padStart(2, '0'),
    value: String(i).padStart(2, '0')
  }))
})

const minuteOptions = computed(() => {
  return Array.from({ length: 60 }, (_, i) => ({
    label: String(i).padStart(2, '0'),
    value: String(i).padStart(2, '0')
  }))
})

function selectHour(hour) {
  tempSelectedHour.value = hour
  nextTick(() => {
    if (hourScrollRef.value) {
      const element = hourScrollRef.value.querySelector(`[data-hour="${hour}"]`)
      if (element) {
        element.scrollIntoView({ block: 'start', behavior: 'smooth' })
      }
    }
  })
}

function selectMinute(minute) {
  tempSelectedMinute.value = minute
  nextTick(() => {
    if (minuteScrollRef.value) {
      const element = minuteScrollRef.value.querySelector(`[data-minute="${minute}"]`)
      if (element) {
        element.scrollIntoView({ block: 'start', behavior: 'smooth' })
      }
    }
  })
}

function calculatePosition() {
  nextTick(() => {
    setTimeout(() => {
      if (!dropdownRef.value || !triggerRef.value) return

      const trigger = triggerRef.value
      const triggerRect = trigger.getBoundingClientRect()
      const viewportWidth = window.innerWidth
      const viewportHeight = window.innerHeight

      // 获取下拉框的实际尺寸
      const dropdownWidth = dropdownRef.value.offsetWidth
      const dropdownHeight = dropdownRef.value.offsetHeight

      // 计算垂直位置
      const spaceBelow = viewportHeight - triggerRect.bottom
      const spaceAbove = triggerRect.top

      if (spaceBelow >= dropdownHeight + 10) {
        // 显示在下方
        dropdownPosition.value.top = `${triggerRect.bottom + 8}px`
        dropdownPosition.value.bottom = 'auto'
      } else if (spaceAbove >= dropdownHeight + 10) {
        // 显示在上方
        dropdownPosition.value.top = 'auto'
        dropdownPosition.value.bottom = `${viewportHeight - triggerRect.top + 8}px`
      } else {
        // 空间都不够，显示在空间更大的一侧
        if (spaceBelow >= spaceAbove) {
          dropdownPosition.value.top = `${triggerRect.bottom + 8}px`
          dropdownPosition.value.bottom = 'auto'
        } else {
          dropdownPosition.value.top = 'auto'
          dropdownPosition.value.bottom = `${viewportHeight - triggerRect.top + 8}px`
        }
      }

      // 计算水平位置
      const spaceRight = viewportWidth - triggerRect.right
      const spaceLeft = triggerRect.left

      if (triggerRect.left + dropdownWidth <= viewportWidth - 10) {
        // 左对齐，右侧空间足够
        dropdownPosition.value.left = `${triggerRect.left}px`
        dropdownPosition.value.right = 'auto'
      } else if (triggerRect.right - dropdownWidth >= 10) {
        // 右对齐，左侧空间足够
        dropdownPosition.value.left = 'auto'
        dropdownPosition.value.right = `${viewportWidth - triggerRect.right}px`
      } else {
        // 居中显示
        const leftPos = Math.max(10, Math.min(viewportWidth - dropdownWidth - 10, triggerRect.left))
        dropdownPosition.value.left = `${leftPos}px`
        dropdownPosition.value.right = 'auto'
      }

      // 计算时间选择器高度
      calculateTimePickerHeight()

      // 滚动到当前选中的时间
      scrollToSelectedTime()
    }, 50)
  })
}

function scrollToSelectedTime() {
  nextTick(() => {
    if (tempSelectedHour.value && hourScrollRef.value) {
      const element = hourScrollRef.value.querySelector(`[data-hour="${tempSelectedHour.value}"]`)
      if (element) {
        element.scrollIntoView({ block: 'start', behavior: 'smooth' })
      }
    }
    if (tempSelectedMinute.value && minuteScrollRef.value) {
      const element = minuteScrollRef.value.querySelector(`[data-minute="${tempSelectedMinute.value}"]`)
      if (element) {
        element.scrollIntoView({ block: 'start', behavior: 'smooth' })
      }
    }
  })
}

function calculateTimePickerHeight() {
  nextTick(() => {
    if (calendarRef.value) {
      const calendarHeight = calendarRef.value.offsetHeight
      timePickerHeight.value = `${calendarHeight}px`
    }
  })
}

function toggleDropdown() {
  if (props.disabled) return
  isOpen.value = !isOpen.value
  if (isOpen.value) {
    initValue()
    nextTick(() => {
      calculatePosition()
    })
  }
}

function closeDropdown() {
  isOpen.value = false
  showYearPicker.value = false
}

function confirm() {
  selectedDate.value = tempSelectedDate.value
  selectedHour.value = tempSelectedHour.value
  selectedMinute.value = tempSelectedMinute.value
  emitValue()
  closeDropdown()
}

function cancel() {
  closeDropdown()
}

function clearValue() {
  selectedDate.value = ''
  selectedHour.value = ''
  selectedMinute.value = ''
  tempSelectedDate.value = ''
  tempSelectedHour.value = ''
  tempSelectedMinute.value = ''
  emit('update:model-value', '')
  emit('update:modelValue', '')
  emit('change', '')
  closeDropdown()
}

// 选择今日/此刻
function selectNowOrToday() {
  const now = dayjs()
  if (props.mode === 'date') {
    selectedDate.value = now.format('YYYY-MM-DD')
    tempSelectedDate.value = selectedDate.value
  } else if (props.mode === 'time') {
    selectedHour.value = now.format('HH')
    selectedMinute.value = now.format('mm')
    tempSelectedHour.value = selectedHour.value
    tempSelectedMinute.value = selectedMinute.value
  } else {
    selectedDate.value = now.format('YYYY-MM-DD')
    selectedHour.value = now.format('HH')
    selectedMinute.value = now.format('mm')
    tempSelectedDate.value = selectedDate.value
    tempSelectedHour.value = selectedHour.value
    tempSelectedMinute.value = selectedMinute.value
  }
  closeDropdown()
  emitValue()
}

function emitValue() {
  let value = ''
  if (props.mode === 'date') {
    value = selectedDate.value
  } else if (props.mode === 'time') {
    if (selectedHour.value && selectedMinute.value) {
      value = `${selectedHour.value}:${selectedMinute.value}`
    }
  } else {
    if (selectedDate.value && selectedHour.value && selectedMinute.value) {
      value = `${selectedDate.value}T${selectedHour.value}:${selectedMinute.value}`
    }
  }
  emit('update:modelValue', value)
  emit('update:model-value', value)
  emit('change', value)
}

function handleResize() {
  if (isOpen.value) {
    calculatePosition()
  }
}

// 点击外部关闭下拉框
function handleClickOutside(event) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target) && 
      triggerRef.value && !triggerRef.value.contains(event.target)) {
    closeDropdown()
  }
}

onMounted(() => {
  window.addEventListener('resize', handleResize)
  window.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', handleResize)
  window.removeEventListener('click', handleClickOutside)
})

defineExpose({
  toggleDropdown,
  closeDropdown
})
</script>

<template>
  <div class="datetime-picker relative">
    <!-- 触发器 -->
    <div ref="triggerRef">
      <slot :toggle="toggleDropdown" :has-value="hasValue" :display-value="displayValue" :clear="clearValue">
        <button @click="toggleDropdown" :disabled="disabled"
          class="datetime-trigger px-2 py-1.5 border border-base-200 rounded-md hover:bg-base-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5 whitespace-nowrap"
          :class="{ 'bg-primary/10 border-primary/30 text-primary': hasValue }">
          <Clock :size="14" />
          <span class="text-xs">{{ displayValue }}</span>
          <X v-if="hasValue && !disabled && clearable" @click.stop="clearValue" :size="12"
            class="ml-auto opacity-60 hover:opacity-100" />
          <svg v-else xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            class="ml-auto opacity-60">
            <path d="m6 9 6 6 6-6" />
          </svg>
        </button>
      </slot>
    </div>

    <!-- 下拉面板 -->

        <teleport to="body">

          <div

            v-if="isOpen"

            ref="dropdownRef"

            class="dropdown-panel fixed bg-base-100 border border-base-200 rounded-lg shadow-lg z-[9999]"

            :style="{

              top: dropdownPosition.top,

              bottom: dropdownPosition.bottom,

              left: dropdownPosition.left,

              right: dropdownPosition.right

            }"

          >
        <div class="flex flex-col max-w-[600px]">
          <!-- 主内容区 -->
          <div class="flex">
            <!-- 日历区域 -->
            <div ref="calendarRef" v-if="mode === 'date' || mode === 'datetime'" class="p-2 min-w-[220px]">
              <!-- 年月导航 -->
              <div class="flex items-center justify-between mb-2">
                <div class="flex items-center gap-0.5">
                  <button @click="prevYear" class="p-0.5 hover:bg-base-200 rounded transition-colors" title="上一年">
                    <ChevronsLeft :size="14" />
                  </button>
                  <button @click="prevMonth" class="p-0.5 hover:bg-base-200 rounded transition-colors" title="上个月">
                    <ChevronLeft :size="14" />
                  </button>
                </div>

                <button @click="toggleYearPicker"
                  class="text-xs font-medium hover:bg-base-200 px-2 py-0.5 rounded transition-colors">
                  {{ currentYear }}年{{ currentMonth }}月
                </button>

                <div class="flex items-center gap-0.5">
                  <button @click="nextMonth" class="p-0.5 hover:bg-base-200 rounded transition-colors" title="下个月">
                    <ChevronRight :size="14" />
                  </button>
                  <button @click="nextYear" class="p-0.5 hover:bg-base-200 rounded transition-colors" title="下一年">
                    <ChevronsRight :size="14" />
                  </button>
                </div>
              </div>

              <!-- 年份选择面板 -->
              <div v-if="showYearPicker" class="mb-2">
                <div class="grid grid-cols-4 gap-1">
                  <button v-for="year in displayedYears" :key="year" @click="selectYear(year)"
                    class="text-[10px] py-1 hover:bg-base-200 rounded transition-colors"
                    :class="{ 'bg-primary text-primary-content': year === currentYear }">
                    {{ year }}
                  </button>
                </div>
                <div class="flex justify-between mt-2">
                  <button @click="prevYearPage" class="text-[10px] hover:bg-base-200 px-2 py-0.5 rounded">
                    上一页
                  </button>
                  <button @click="nextYearPage" class="text-[10px] hover:bg-base-200 px-2 py-0.5 rounded">
                    下一页
                  </button>
                </div>
              </div>

              <!-- 星期标题 -->
              <div v-else class="grid grid-cols-7 gap-0.5 mb-1">
                <div v-for="day in ['日', '一', '二', '三', '四', '五', '六']" :key="day"
                  class="text-center text-[10px] text-base-content/50 py-0.5">
                  {{ day }}
                </div>
              </div>

              <!-- 日期格子 -->
              <div v-if="!showYearPicker" class="grid grid-cols-7 gap-0.5">
                <div v-for="day in calendarDays" :key="day.dateStr" @click="selectDate(day.dateStr)"
                  class="aspect-square flex items-center justify-center text-[10px] rounded cursor-pointer transition-colors"
                  :class="{
                    'text-base-content/30': !day.isCurrentMonth,
                    'bg-primary text-primary-content': day.isSelected && day.isCurrentMonth,
                    'bg-primary/10 text-primary': day.isSelected && !day.isCurrentMonth,
                    'text-primary font-medium': day.isToday && !day.isSelected,
                    'hover:bg-base-200': !day.isSelected
                  }">
                  {{ day.day }}
                </div>
              </div>
            </div>

            <!-- 时间选择区域 -->
            <div v-if="mode === 'time' || mode === 'datetime'"
              class="p-2 border-l border-base-200 min-w-[100px] w-[120px] flex flex-col"
              :style="{ height: timePickerHeight }">
              <!-- 时间显示标题 -->
              <div class="text-[10px] text-base-content/60 mb-1.5 flex-shrink-0">
                {{ (tempSelectedHour && tempSelectedMinute) ? `${tempSelectedHour}:${tempSelectedMinute}` : '选择时间' }}
              </div>
              <!-- 时间选择列表 -->
              <div class="flex gap-1.5 flex-1 min-h-0">
                <!-- 小时 -->
                <div class="flex-1 border border-base-200 rounded overflow-hidden">
                  <div ref="hourScrollRef" class="overflow-y-auto custom-scrollbar h-full">
                    <div v-for="hour in hourOptions" :key="hour.value" :data-hour="hour.value" @click="selectHour(hour.value)"
                      class="text-[10px] py-1.5 px-1 hover:bg-base-200 cursor-pointer text-center transition-colors"
                      :class="{ 'bg-primary/20 text-primary font-medium': tempSelectedHour === hour.value }">
                      {{ hour.label }}
                    </div>
                  </div>
                </div>
                <!-- 分钟 -->
                <div class="flex-1 border border-base-200 rounded overflow-hidden">
                  <div ref="minuteScrollRef" class="overflow-y-auto custom-scrollbar h-full">
                    <div v-for="minute in minuteOptions" :key="minute.value" :data-minute="minute.value" @click="selectMinute(minute.value)"
                      class="text-[10px] py-1.5 px-1 hover:bg-base-200 cursor-pointer text-center transition-colors"
                      :class="{ 'bg-primary/20 text-primary font-medium': tempSelectedMinute === minute.value }">
                      {{ minute.label }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部按钮 -->
          <div class="flex items-center justify-between px-3 py-2 border-t border-base-200">
            <div v-if="mode !== 'time'" class="flex items-center gap-2">
              <button
                v-if="hasValue && clearable"
                @click="clearValue"
                class="px-3 py-1 text-xs text-error hover:bg-error/10 rounded transition-colors"
              >
                清空
              </button>
              <button @click="selectNowOrToday"
                class="px-3 py-1 text-xs text-primary hover:bg-primary/10 rounded transition-colors">
                {{ mode === 'datetime' ? '此刻' : '今日' }}
              </button>
            </div>
            <div v-else></div>
            <div class="flex items-center gap-2">
              <button @click="cancel"
                class="px-3 py-1 text-xs border border-base-200 rounded hover:bg-base-200 transition-colors">
                取消
              </button>
              <button @click="confirm"
                class="px-3 py-1 text-xs bg-primary text-primary-content rounded hover:bg-primary/90 transition-colors">
                确认
              </button>
            </div>
          </div>
        </div>
      </div>
    </teleport>
  </div>
</template>

<style scoped>
.datetime-picker {
  position: relative;
}

.datetime-trigger {
  user-select: none;
}

.dropdown-panel {
  animation: fadeIn 0.15s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.5);
  border-radius: 2px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: rgba(156, 163, 175, 0.7);
}
</style>
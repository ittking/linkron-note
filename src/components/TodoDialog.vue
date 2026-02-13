<script setup>
import { ref, watch, computed } from 'vue'
import dayjs from 'dayjs'
import Button from './ui/Button.vue'
import Toggle from './ui/Toggle.vue'
import Select from './ui/Select.vue'
import DateTimePicker from './ui/DateTimePicker.vue'

dayjs.locale('zh-cn')

const props = defineProps({
  show: {
    type: Boolean,
    default: false
  },
  date: {
    type: String,
    default: ''
  },
  todo: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['update:show', 'save', 'delete'])

// 表单数据
const newTodoText = ref('')
const formStatus = ref('todo')
const formReminderEnabled = ref(false)
const formReminderType = ref('once')
const formReminderTime = ref('')
const formRepeatTime = ref('')
const formRepeatType = ref('day')
const formRepeatInterval = ref(1)
const formRepeatWeekdays = ref([1, 3, 5])
const formRepeatMonthDays = ref([1])
const formRepeatYearMonth = ref(1)
const formRepeatYearDay = ref(1)

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
  { value: 'weekday', label: '按星期' },
  { value: 'month', label: '按月' },
  { value: 'year', label: '按年' }
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

// 月份选项
const monthOptions = Array.from({ length: 12 }, (_, i) => ({
  label: `${i + 1}月`,
  value: i + 1
}))

// 日期选项
const dayOptions = Array.from({ length: 31 }, (_, i) => ({
  label: `${i + 1}日`,
  value: i + 1
}))

// 是否是编辑模式
const isEditing = computed(() => !!props.todo)

// 重置表单
function resetForm() {
  newTodoText.value = ''
  formStatus.value = 'todo'
  formReminderEnabled.value = false
  formReminderType.value = 'once'
  formReminderTime.value = ''
  formRepeatTime.value = ''
  formRepeatType.value = 'day'
  formRepeatInterval.value = 1
  formRepeatWeekdays.value = [1, 3, 5]
  formRepeatMonthDays.value = [1]
  formRepeatYearMonth.value = 1
  formRepeatYearDay.value = 1
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

// 切换月份日期选择
function toggleMonthDay(value) {
  const index = formRepeatMonthDays.value.indexOf(value)
  if (index > -1) {
    formRepeatMonthDays.value.splice(index, 1)
  } else {
    formRepeatMonthDays.value.push(value)
  }
}

// 监听 show 和 todo 变化
watch(() => props.show, (newVal) => {
  if (newVal && props.todo) {
    // 编辑模式
    newTodoText.value = props.todo.text
    formStatus.value = props.todo.status || 'todo'

    if (props.todo.reminder) {
      formReminderEnabled.value = true
      const reminder = props.todo.reminder
      formReminderType.value = reminder.reminder_type === 'onetime' ? 'once' : (reminder.reminder_type === 'repeat' ? 'repeat' : 'none')
      
      // 一次性提醒：使用 repeat_time（完整日期时间）
      formReminderTime.value = reminder.repeat_time || ''
      
      // 重复提醒：使用 repeat_time（只有 HH:mm），需要转换为完整格式供 time 模式使用
      if (reminder.reminder_type === 'repeat' && reminder.repeat_time) {
        const timeValue = reminder.repeat_time
        // 检查是否已经是完整格式
        if (timeValue.includes('T') || timeValue.includes('-')) {
          // 已经是完整格式，直接使用
          formRepeatTime.value = timeValue
        } else {
          // 只有 HH:mm，补全为完整格式
          formRepeatTime.value = `${dayjs().format('YYYY-MM-DD')}T${timeValue}`
        }
      } else {
        formRepeatTime.value = ''
      }

      if (reminder.reminder_type === 'repeat' && reminder.repeat_rule) {
        const rule = reminder.repeat_rule
        formRepeatType.value = rule

        if (rule === 'day') {
          formRepeatInterval.value = reminder.repeat_interval || 1
        } else if (rule === 'weekday') {
          formRepeatWeekdays.value = [reminder.repeat_day_of_week || 1]
        } else if (rule === 'month') {
          formRepeatMonthDays.value = [reminder.repeat_day_of_month || 1]
        } else if (rule === 'year') {
          formRepeatYearMonth.value = reminder.repeat_month || 1
          formRepeatYearDay.value = reminder.repeat_day_of_month || 1
        }
      }
    } else {
      formReminderEnabled.value = false
    }
  } else if (newVal) {
    // 新增模式
    resetForm()
  }
})

// 关闭对话框
function closeDialog() {
  emit('update:show', false)
  resetForm()
}

// 保存待办
function saveTodo() {
  if (!newTodoText.value.trim()) return

  // 构建提醒配置
  let reminder = null
  if (formReminderEnabled.value) {
    if (formReminderType.value === 'once') {
      reminder = JSON.stringify({
        reminder_type: 'onetime',
        repeat_time: formReminderTime.value // 一次性提醒：完整日期时间
      })
    } else if (formReminderType.value === 'repeat') {
      // 重复提醒：将 HH:mm 转换为今日的完整日期时间
      const timeOnly = formRepeatTime.value || ''
      const fullDateTime = timeOnly ? `${dayjs().format('YYYY-MM-DD')}T${timeOnly}` : ''
      reminder = JSON.stringify({
        reminder_type: 'repeat',
        repeat_time: fullDateTime,
        repeat_rule: formRepeatType.value,
        repeat_interval: formRepeatType.value === 'day' ? formRepeatInterval.value : undefined,
        repeat_day_of_week: formRepeatType.value === 'weekday' ? formRepeatWeekdays.value[0] : undefined,
        repeat_day_of_month: formRepeatType.value === 'month' ? formRepeatMonthDays.value[0] : undefined,
        repeat_month: formRepeatType.value === 'year' ? formRepeatYearMonth.value : undefined
      })
    }
  }

  emit('save', {
    id: props.todo?.id,
    date: props.date,
    text: newTodoText.value,
    status: formStatus.value,
    reminder
  })

  closeDialog()
}

// 删除待办
function deleteTodo() {
  if (!props.todo) return
  emit('delete', props.todo.id)
  closeDialog()
}
</script>

<template>
  <div v-if="show"
    class="fixed inset-0 bg-base-content/20 backdrop-blur-sm flex items-center justify-center z-50"
    @click.self="closeDialog">
    <div class="bg-base-100 rounded-lg shadow-xl p-6 w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto">
      <h3 class="text-lg font-semibold text-base-content mb-4">
        {{ isEditing ? '编辑待办' : '添加待办' }} - {{ date }}
      </h3>

      <!-- 内容输入 -->
      <textarea v-model="newTodoText" @keyup.enter.exact="saveTodo" @keyup.enter.shift.exact.prevent
        placeholder="输入待办事项..."
        class="w-full min-h-[80px] px-3 py-2 border border-primary/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent resize-none text-sm mb-4"></textarea>

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
            <DateTimePicker
              v-model="formReminderTime"
              mode="datetime"
              :min="dayjs().format('YYYY-MM-DDTHH:mm')"
              :clearable="true"
              placeholder="选择提醒时间"
            />
          </div>

          <!-- 重复提醒 -->
          <div v-if="formReminderType === 'repeat'" class="space-y-3">
            <label class="block text-xs text-base-content/60 mb-1.5">重复规则</label>
            <div class="flex flex-wrap gap-2 mb-2">
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

            <!-- 按月重复 -->
            <div v-if="formRepeatType === 'month'">
              <label class="block text-xs text-base-content/60 mb-1.5">每月</label>
              <div class="flex flex-wrap gap-1.5">
                <button v-for="d in 31" :key="d" @click="toggleMonthDay(d)"
                  class="w-8 h-8 rounded text-xs border transition-colors"
                  :class="formRepeatMonthDays.includes(d) ? 'bg-primary text-primary-content border-primary' : 'bg-base-100 text-base-content/50 border-base-200'">
                  {{ d }}
                </button>
              </div>
            </div>

            <!-- 按年重复 -->
            <div v-if="formRepeatType === 'year'" class="space-y-2">
              <div class="flex items-center gap-2">
                <span class="text-xs text-base-content/60">每年</span>
                <Select
                  v-model="formRepeatYearMonth"
                  :options="monthOptions"
                  placeholder="选择月份"
                  size="sm"
                  class="flex-1"
                />
                <Select
                  v-model="formRepeatYearDay"
                  :options="dayOptions"
                  placeholder="选择日期"
                  size="sm"
                  class="flex-1"
                />
              </div>
            </div>

            <!-- 重复提醒时间 -->
            <div>
              <label class="block text-xs text-base-content/60 mb-1.5">提醒时间</label>
              <DateTimePicker
                v-model="formRepeatTime"
                mode="time"
                placeholder="选择提醒时间"
              />
            </div>
          </div>
        </div>
      </div>

      <div class="flex justify-between items-center mt-6">
        <Button v-if="isEditing" variant="ghost" size="sm" @click="deleteTodo" class="text-error hover:text-error hover:bg-error/10">
          删除
        </Button>
        <div v-else></div>
        <div class="flex gap-2">
          <Button variant="secondary" size="sm" @click="closeDialog">
            取消
          </Button>
          <Button variant="primary" size="sm" @click="saveTodo">
            {{ isEditing ? '保存' : '添加' }}
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
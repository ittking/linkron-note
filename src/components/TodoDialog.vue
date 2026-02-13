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
const formRepeatWeekdays = ref([1])  // 默认选中周一
const formRepeatMonthDays = ref([1])
const formRepeatYearMonth = ref(1)
const formRepeatYearDay = ref(1)

// 错误提示
const todoTextError = ref('')
const reminderTimeError = ref('')

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

// 每隔天数选项（1-31天）
const intervalDayOptions = Array.from({ length: 31 }, (_, i) => ({
  label: `${i + 1}天`,
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
  formRepeatWeekdays.value = [1]  // 默认选中周一
  formRepeatMonthDays.value = [1]
  formRepeatYearMonth.value = 1
  formRepeatYearDay.value = 1
  todoTextError.value = ''  // 清除错误提示
  reminderTimeError.value = ''  // 清除错误提示
}

// 切换星期选择
function toggleWeekday(value) {
  const index = formRepeatWeekdays.value.indexOf(value)
  if (index > -1) {
    // 如果是最后一个选中的，不允许取消
    if (formRepeatWeekdays.value.length === 1) {
      return
    }
    formRepeatWeekdays.value.splice(index, 1)
  } else {
    formRepeatWeekdays.value.push(value)
  }
}

// 切换月份日期选择
function toggleMonthDay(value) {
  const index = formRepeatMonthDays.value.indexOf(value)
  if (index > -1) {
    // 如果是最后一个选中的，不允许取消
    if (formRepeatMonthDays.value.length === 1) {
      return
    }
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
      formReminderType.value = reminder.type === 'onetime' ? 'once' : (reminder.type === 'repeat' ? 'repeat' : 'none')
      
      // 一次性提醒：使用 repeatTime（完整日期时间）
      formReminderTime.value = reminder.repeatTime || ''
      
      // 重复提醒：使用 repeatTime（只有 HH:mm），需要转换为完整格式供 time 模式使用
      if (reminder.type === 'repeat' && reminder.repeatTime) {
        const timeValue = reminder.repeatTime
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

      if (reminder.type === 'repeat' && reminder.repeatRule) {
        const rule = reminder.repeatRule
        formRepeatType.value = rule

        if (rule === 'day') {
          formRepeatInterval.value = reminder.repeatInterval || 1
        } else if (rule === 'weekday') {
          formRepeatWeekdays.value = Array.isArray(reminder.repeatDayOfWeek) ? reminder.repeatDayOfWeek : [reminder.repeatDayOfWeek || 1]
        } else if (rule === 'month') {
          formRepeatMonthDays.value = Array.isArray(reminder.repeatDayOfMonth) ? reminder.repeatDayOfMonth : [reminder.repeatDayOfMonth || 1]
        } else if (rule === 'year') {
          formRepeatYearMonth.value = reminder.repeatMonth || 1
          formRepeatYearDay.value = reminder.repeatDayOfMonth || 1
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

// 监听待办内容变化，清除错误提示
watch(newTodoText, () => {
  if (newTodoText.value.trim()) {
    todoTextError.value = ''
  }
})

// 监听提醒时间变化，清除错误提示
watch([formReminderTime, formRepeatTime], () => {
  if (formReminderType.value === 'once' && formReminderTime.value.trim()) {
    reminderTimeError.value = ''
  } else if (formReminderType.value === 'repeat' && formRepeatTime.value.trim()) {
    reminderTimeError.value = ''
  }
})

// 关闭对话框
function closeDialog() {
  emit('update:show', false)
  resetForm()
}

// 保存待办
function saveTodo() {
  // 清除之前的错误
  todoTextError.value = ''
  reminderTimeError.value = ''

  // 验证待办内容
  if (!newTodoText.value.trim()) {
    todoTextError.value = '请输入待办内容'
    return
  }

  // 如果开启了提醒设置，验证提醒时间
  if (formReminderEnabled.value) {
    const reminderTime = formReminderType.value === 'once' ? formReminderTime.value : formRepeatTime.value
    if (!reminderTime || !reminderTime.trim()) {
      reminderTimeError.value = '请设置提醒时间'
      return
    }
  }

  // 构建提醒配置
  let reminder = null
  if (formReminderEnabled.value) {
    if (formReminderType.value === 'once') {
      reminder = JSON.stringify({
        type: 'onetime',
        repeatTime: formReminderTime.value // 一次性提醒：完整日期时间
      })
    } else if (formReminderType.value === 'repeat') {
      // 重复提醒：将时间转换为今日的完整日期时间
      const timeInput = formRepeatTime.value || ''
      let timeOnly = ''
      
      // 检查是否已经是完整格式
      if (timeInput.includes('T')) {
        // 已经是完整格式，提取 HH:mm 部分
        const dateObj = dayjs(timeInput)
        if (dateObj.isValid()) {
          timeOnly = dateObj.format('HH:mm')
        }
      } else {
        // 只有 HH:mm 格式，直接使用
        timeOnly = timeInput
      }
      
      const fullDateTime = timeOnly ? `${dayjs().format('YYYY-MM-DD')}T${timeOnly}` : ''
      reminder = JSON.stringify({
        type: 'repeat',
        repeatTime: fullDateTime,
        repeatRule: formRepeatType.value,
        repeatInterval: formRepeatType.value === 'day' ? formRepeatInterval.value : undefined,
        repeatDayOfWeek: formRepeatType.value === 'weekday' ? formRepeatWeekdays.value : undefined,
        repeatDayOfMonth: formRepeatType.value === 'month' ? formRepeatMonthDays.value : undefined,
        repeatMonth: formRepeatType.value === 'year' ? formRepeatYearMonth.value : undefined
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
      <textarea v-model="newTodoText"
        placeholder="输入待办事项..."
        class="w-full min-h-[80px] px-3 py-2 border border-primary/50 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent resize-none text-sm"
        :class="{ 'border-error ring-error': todoTextError }"></textarea>
      <div v-if="todoTextError" class="text-xs text-error mt-1">{{ todoTextError }}</div>

      <!-- 状态选择 -->
      <div class="my-4">
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
            <div v-if="reminderTimeError && formReminderType === 'once'" class="text-xs text-error mt-1">{{ reminderTimeError }}</div>
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
            <div v-if="formRepeatType === 'day'">
              <label class="block text-xs text-base-content/60 mb-1.5">重复间隔</label>
              <Select
                v-model="formRepeatInterval"
                :options="intervalDayOptions"
                placeholder="选择间隔天数"
                size="sm"
              />
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
              <div v-if="reminderTimeError && formReminderType === 'repeat'" class="text-xs text-error mt-1">{{ reminderTimeError }}</div>
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
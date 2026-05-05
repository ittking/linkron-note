import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { useWorkDirectory } from './useWorkDirectory'
import { useToast } from './useToast'
import dayjs from 'dayjs'

export function useReminder() {
  const reminders = ref([])
  const isChecking = ref(false)
  const { getWorkDirectory } = useWorkDirectory()

  // 请求通知权限
  async function requestNotificationPermission() {
    try {
      let permissionGranted = await isPermissionGranted()

      if (!permissionGranted) {
        const permission = await requestPermission()
        permissionGranted = permission === 'granted'
      }

      return permissionGranted
    } catch (error) {
      console.error('Failed to request notification permission:', error)
      return false
    }
  }

  // 判断是否应该发送一次性提醒
  function shouldSendOneTimeReminder(reminder, currentTime) {
    if (!reminder.repeatTime) return false

    const reminderTime = dayjs(reminder.repeatTime)
    if (!reminderTime.isValid()) return false

    // 检查年月日时分是否精确匹配
    const reminderDateTime = reminderTime.format('YYYY-MM-DD HH:mm')
    const currentDateTime = currentTime.format('YYYY-MM-DD HH:mm')

    // 只有在秒数为0且时间匹配时才提醒
    return currentTime.second() === 0 && reminderDateTime === currentDateTime
  }

  // 判断是否应该发送重复提醒
  function shouldSendRepeatReminder(reminder, todo, currentTime) {
    if (!reminder.repeatTime) return false

    const reminderTime = dayjs(reminder.repeatTime)
    if (!reminderTime.isValid()) return false

    // 检查时分是否匹配
    const reminderHourMinute = reminderTime.format('HH:mm')
    const currentHourMinute = currentTime.format('HH:mm')

    if (reminderHourMinute !== currentHourMinute) return false

    const todoDate = dayjs(todo.date)
    if (!todoDate.isValid()) return false

    const currentDate = dayjs()
    const rule = reminder.repeatRule

    // 根据重复规则判断是否应该提醒
    let shouldRemind = false

    if (rule === 'day') {
      const interval = reminder.repeatInterval || 1
      const daysDiff = currentDate.diff(todoDate, 'day')
      shouldRemind = daysDiff >= 0 && daysDiff % interval === 0
    } else if (rule === 'weekday') {
      const weekdays = reminder.repeatDayOfWeek
      const currentWeekday = currentDate.day()
      if (Array.isArray(weekdays)) {
        shouldRemind = weekdays.includes(currentWeekday)
      } else {
        shouldRemind = currentWeekday === weekdays
      }
    } else if (rule === 'month') {
      const days = reminder.repeatDayOfMonth
      const currentDay = currentDate.date()
      if (Array.isArray(days)) {
        shouldRemind = days.includes(currentDay)
      } else {
        shouldRemind = currentDay === days
      }
    } else if (rule === 'year') {
      const month = reminder.repeatMonth
      const day = reminder.repeatDayOfMonth
      const currentMonth = currentDate.month() + 1
      const currentDay = currentDate.date()
      shouldRemind = currentMonth === month && currentDay === day
    }

    // 只有在秒数为0且应该提醒时才返回true
    return currentTime.second() === 0 && shouldRemind
  }

  // 获取需要提醒的待办事项
  async function fetchReminders() {
    if (isChecking.value) return

    try {
      isChecking.value = true
      const workDirectory = await getWorkDirectory()
      const result = await invoke('get_reminders', { workDirectory })
      reminders.value = result

      const currentTime = dayjs()

      debugCount++
      if (!result || result.length === 0) {
        if (debugCount === 1 || debugCount % 60 === 0) {
          console.log('[Reminder] No reminders found (checked', debugCount, 'times), result:', JSON.stringify(result), 'currentTime:', currentTime.format('MM-DD HH:mm:ss'))
        }
        return
      }

      debugCount = 0
      console.log('[Reminder] Found', result.length, 'todos with reminders')
      // 发送通知
      for (const todo of reminders.value) {
        if (!todo.reminder) continue

        const reminder = todo.reminder
        let shouldNotify = false

        if (reminder.type === 'onetime') {
          shouldNotify = shouldSendOneTimeReminder(reminder, currentTime)
          console.log('[Reminder] OneTime check:', todo.text, 'shouldNotify:', shouldNotify, 'reminderTime:', reminder.repeatTime, 'current:', currentTime.format('YYYY-MM-DD HH:mm'))
        } else if (reminder.type === 'repeat') {
          shouldNotify = shouldSendRepeatReminder(reminder, todo, currentTime)
          console.log('[Reminder] Repeat check:', todo.text, 'shouldNotify:', shouldNotify, 'reminderTime:', reminder.repeatTime, 'current:', currentTime.format('HH:mm'))
        }

        if (shouldNotify) {
          console.log('[Reminder] Triggering notification for:', todo.text)
          const { showToast } = useToast()

          // 系统通知（macOS 可能不显示）
          if (permissionGranted) {
            try {
              await sendNotification({
                title: '待办提醒',
                body: `${todo.text} - ${currentTime.format('HH:mm')}`
              })
            } catch (e) {
              console.error('[Reminder] sendNotification failed:', e)
            }
          }

          // 兜底：应用内 Toast 提示
          showToast(`⏰ ${todo.text}`, 'info')
          console.log('[Reminder] Toast shown for:', todo.text)
        }
      }
    } catch (error) {
      console.error('Failed to fetch reminders:', error)
    } finally {
      isChecking.value = false
    }
  }

  // 启动提醒检查任务
  let reminderInterval = null
  let permissionGranted = false
  let debugCount = 0

  async function startReminderCheck() {
    // 先请求权限
    permissionGranted = await requestNotificationPermission()
    console.log('[Reminder] Notification permission:', permissionGranted)

    if (!permissionGranted) {
      console.warn('[Reminder] Notification permission not granted, reminders will only log to console')
    }

    // 每秒检查一次
    reminderInterval = setInterval(() => {
      fetchReminders()
    }, 1000)
  }

  // 停止提醒检查任务
  function stopReminderCheck() {
    if (reminderInterval) {
      clearInterval(reminderInterval)
      reminderInterval = null
    }
  }

  return {
    reminders,
    isChecking,
    requestNotificationPermission,
    fetchReminders,
    startReminderCheck,
    stopReminderCheck
  }
}
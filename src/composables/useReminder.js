import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { useWorkDirectory } from './useWorkDirectory'

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

      // 发送启动通知
      if (permissionGranted) {
        await sendNotification({
          title: 'iFlow 启动',
          body: '消息通知已启动，待办事项提醒将正常工作'
        })
      }

      return permissionGranted
    } catch (error) {
      console.error('Failed to request notification permission:', error)
      return false
    }
  }

  // 获取需要提醒的待办事项
  async function fetchReminders() {
    if (isChecking.value) return

    try {
      isChecking.value = true
      const workDirectory = await getWorkDirectory()
      console.log('获取工作目录:', workDirectory)
      const result = await invoke('get_reminders', { workDirectory })
      console.log('获取到的提醒数据 (原始):', result)
      console.log('数据类型:', typeof result)
      console.log('是否为数组:', Array.isArray(result))
      console.log('数组长度:', result ? result.length : 'N/A')
      reminders.value = result

      // 发送通知
      for (const reminder of reminders.value) {
        console.log('发送通知:', reminder)
        await sendNotification({
          title: '待办提醒',
          body: `${new Date().toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })} - ${reminder.text}`
        })
      }
    } catch (error) {
      console.error('Failed to fetch reminders:', error)
    } finally {
      isChecking.value = false
    }
  }

  // 启动提醒检查任务
  let reminderInterval = null

  function startReminderCheck() {
    // 先请求权限
    requestNotificationPermission()

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
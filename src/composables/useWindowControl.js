import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, currentMonitor as getCurrentMonitor } from '@tauri-apps/api/window'

/**
 * 窗口控制 Composable
 * 实现窗口放大和缩小功能，支持多显示器环境
 * 根据操作系统类型使用不同的实现方式
 */
export function useWindowControl() {
  const isFullscreen = ref(false)
  const currentOS = ref('unknown')
  const previousWindowState = ref({
    width: 0,
    height: 0,
    x: 0,
    y: 0,
    resizable: false
  })

  /**
   * 获取当前窗口
   */
  async function getCurrentWebviewWindow() {
    return getCurrentWindow()
  }

  /**
   * 初始化，获取操作系统类型
   */
  async function init() {
    try {
      currentOS.value = await invoke('get_os')
    } catch (error) {
      console.error('获取操作系统类型失败:', error)
    }
  }

  /**
   * 切换窗口全屏状态
   */
  async function toggleFullscreen() {
    const window = await getCurrentWebviewWindow()
    
    if (isFullscreen.value) {
      // 退出全屏
      await exitFullscreen(window)
    } else {
      // 进入全屏
      await enterFullscreen(window)
    }
    
    isFullscreen.value = !isFullscreen.value
  }

  /**
   * 进入全屏模式
   */
  async function enterFullscreen(window) {
    try {
      // macOS 使用手动全屏方式
      if (currentOS.value === 'macos') {
        await enterFullscreenMacOS(window)
      } else {
        // 其他平台使用原生全屏 API
        await enterFullscreenNative(window)
      }
    } catch (error) {
      console.error('进入全屏失败:', error)
      throw error
    }
  }

  /**
   * macOS 专用：手动全屏实现
   * 参考：https://juejin.cn/post/7585006378533453866
   */
  async function enterFullscreenMacOS(window) {
    // 1. 保存当前窗口状态
    const size = await window.innerSize()
    const position = await window.outerPosition()
    const resizable = await window.isResizable()

    previousWindowState.value = {
      width: size.width,
      height: size.height,
      x: position.x,
      y: position.y,
      resizable
    }

    // 2. 获取当前鼠标所在的显示器
    const currentMonitor = await getCurrentMonitor()
    if (!currentMonitor) {
      console.error('无法获取当前显示器信息')
      return
    }

    // 3. 获取缩放因子（用于物理像素到逻辑像素的转换）
    const scaleFactor = await window.scaleFactor()

    // 4. 转换坐标：物理像素 -> 逻辑像素
    const { size: monitorSize, position: monitorPosition } = currentMonitor
    const logicalSize = monitorSize.toLogical(scaleFactor)
    const logicalPosition = monitorPosition.toLogical(scaleFactor)

    // 5. 设置窗口大小和位置，铺满当前显示器
    await window.setSize(logicalSize)
    await window.setPosition(logicalPosition)
    await window.setResizable(true)
  }

  /**
   * 原生全屏实现（Windows、Linux）
   */
  async function enterFullscreenNative(window) {
    // 保存当前窗口状态
    const size = await window.innerSize()
    const position = await window.outerPosition()
    const resizable = await window.isResizable()

    previousWindowState.value = {
      width: size.width,
      height: size.height,
      x: position.x,
      y: position.y,
      resizable
    }

    // 使用原生全屏 API
    await window.setFullscreen(true)
  }

  /**
   * 退出全屏模式
   */
  async function exitFullscreen(window) {
    try {
      // macOS 使用手动恢复方式
      if (currentOS.value === 'macos') {
        await exitFullscreenMacOS(window)
      } else {
        // 其他平台使用原生 API
        await exitFullscreenNative(window)
      }
    } catch (error) {
      console.error('退出全屏失败:', error)
      throw error
    }
  }

  /**
   * macOS 专用：手动恢复全屏
   */
  async function exitFullscreenMacOS(window) {
    const { width, height, x, y, resizable } = previousWindowState.value

    // 恢复窗口大小
    await window.setSize({ width, height })

    // 恢复窗口位置
    await window.setPosition({ x, y })

    // 恢复窗口可调整大小状态
    await window.setResizable(resizable)
  }

  /**
   * 原生退出全屏（Windows、Linux）
   */
  async function exitFullscreenNative(window) {
    await window.setFullscreen(false)

    // 恢复之前的窗口状态
    const { width, height, x, y, resizable } = previousWindowState.value
    
    await window.setSize({ width, height })
    await window.setPosition({ x, y })
    await window.setResizable(resizable)
  }

  /**
   * 设置窗口大小
   */
  async function setWindowSize(width, height) {
    const window = await getCurrentWebviewWindow()
    await window.setSize({ width, height })
  }

  /**
   * 设置窗口位置
   */
  async function setWindowPosition(x, y) {
    const window = await getCurrentWebviewWindow()
    await window.setPosition({ x, y })
  }

  /**
   * 获取窗口大小
   */
  async function getWindowSize() {
    const window = await getCurrentWebviewWindow()
    return await window.innerSize()
  }

  /**
   * 获取窗口位置
   */
  async function getWindowPosition() {
    const window = await getCurrentWebviewWindow()
    return await window.outerPosition()
  }

  /**
   * 设置窗口是否可调整大小
   */
  async function setWindowResizable(resizable) {
    const window = await getCurrentWebviewWindow()
    await window.setResizable(resizable)
  }

  /**
   * 窗口居中
   */
  async function centerWindow() {
    const window = await getCurrentWebviewWindow()
    await window.center()
  }

  /**
   * 最小化窗口
   */
  async function minimizeWindow() {
    const window = await getCurrentWebviewWindow()
    await window.minimize()
  }

  /**
   * 最大化窗口
   */
  async function maximizeWindow() {
    const window = await getCurrentWebviewWindow()
    await window.toggleMaximize()
  }

  /**
   * 关闭窗口
   */
  async function closeWindow() {
    const window = await getCurrentWebviewWindow()
    await window.close()
  }

  // 初始化时获取操作系统类型
  init()

  return {
    isFullscreen,
    currentOS,
    toggleFullscreen,
    enterFullscreen,
    exitFullscreen,
    setWindowSize,
    setWindowPosition,
    getWindowSize,
    getWindowPosition,
    setWindowResizable,
    centerWindow,
    minimizeWindow,
    maximizeWindow,
    closeWindow
  }
}
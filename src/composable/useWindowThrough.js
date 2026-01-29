import { ref, nextTick } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * 窗口穿透管理器
 * 监听鼠标移动事件，根据鼠标是否在指定元素范围内控制窗口穿透状态
 */
export function useWindowThrough() {
  const isListening = ref(false);
  const registeredElements = ref(new Map());
  const unlistenFn = ref(null);
  const currentWindow = ref(null);
  const windowPosition = ref({ x: 0, y: 0 });
  const scaleFactor = ref(1);
  const mutationObserver = ref(null);

  /**
   * 初始化窗口
   */
  async function initWindow() {
    if (!currentWindow.value) {
      currentWindow.value = getCurrentWindow();
    }
  }

  /**
   * 获取窗口位置和缩放比例
   */
  async function updateWindowInfo() {
    try {
      await initWindow();
      const pos = await currentWindow.value.outerPosition();
      const scale = await currentWindow.value.scaleFactor();
      
      windowPosition.value = { x: pos.x, y: pos.y };
      scaleFactor.value = scale;
    } catch (error) {
      console.error("获取窗口信息失败:", error);
    }
  }

  /**
   * 设置窗口穿透状态
   * @param {boolean} ignore - true: 忽略鼠标（穿透），false: 不忽略鼠标（不穿透）
   */
  async function setWindowIgnore(ignore) {
    try {
      await initWindow();
      await currentWindow.value.setIgnoreCursorEvents(ignore);
    } catch (error) {
      console.error("设置窗口穿透失败:", error);
    }
  }

  /**
   * 获取元素的屏幕坐标和尺寸（适配显示器缩放）
   * @param {HTMLElement} element - 目标元素
   * @returns {Object} 包含屏幕坐标和尺寸的对象
   */
  function getElementScreenBounds(element) {
    const rect = element.getBoundingClientRect();
    const scale = scaleFactor.value;
    const winX = windowPosition.value.x;
    const winY = windowPosition.value.y;
    
    // 将元素相对坐标转换为屏幕绝对坐标，并应用缩放比例
    const screenX = winX + rect.left * scale;
    const screenY = winY + rect.top * scale;
    const screenWidth = rect.width * scale;
    const screenHeight = rect.height * scale;
    
    return {
      x: screenX,
      y: screenY,
      width: screenWidth,
      height: screenHeight,
      right: screenX + screenWidth,
      bottom: screenY + screenHeight,
    };
  }

  /**
   * 检查鼠标坐标是否在元素范围内
   * @param {number} mouseX - 鼠标 X 坐标
   * @param {number} mouseY - 鼠标 Y 坐标
   * @param {Object} bounds - 元素的边界信息
   * @returns {boolean} 是否在范围内
   */
  function isMouseInBounds(mouseX, mouseY, bounds) {
    return (
      mouseX >= bounds.x &&
      mouseX <= bounds.right &&
      mouseY >= bounds.y &&
      mouseY <= bounds.bottom
    );
  }

  /**
   * 扫描页面中所有带有 through-listener 属性且值为 true 的元素
   */
  function scanThroughElements() {
    const elements = document.querySelectorAll("[through-listener]");
    registeredElements.value.clear();

    elements.forEach((element) => {
      // 只处理 through-listener 属性值为 true 的元素
      const listenerValue = element.getAttribute("through-listener");
      if (listenerValue !== "true" && listenerValue !== true) {
        return;
      }

      const id =
        element.id ||
        `through-${Date.now()}-${Math.random()}`;
      const bounds = getElementScreenBounds(element);

      registeredElements.value.set(id, {
        element,
        bounds,
        id,
      });
    });
  }

  /**
   * 处理鼠标移动事件
   * @param {Object} event - 鼠标事件对象
   */
  async function handleMouseMove(event) {
    const { x, y } = event.payload;

    // 实时更新窗口位置（处理窗口拖拽后的位置变化）
    await updateWindowInfo();

    // 重新计算所有元素的屏幕边界
    for (const [id, data] of registeredElements.value) {
      data.bounds = getElementScreenBounds(data.element);
    }

    // 检查鼠标是否在任何注册的元素范围内
    let inAnyElement = false;
    for (const [id, data] of registeredElements.value) {
      if (isMouseInBounds(x, y, data.bounds)) {
        inAnyElement = true;
        break;
      }
    }

    // 设置窗口穿透状态
    // 在元素范围内：不穿透（可以交互）
    // 不在元素范围内：穿透（点击穿透到下方窗口）
    await setWindowIgnore(!inAnyElement);
  }

  /**
   * 启动监听
   */
  async function startListening() {
    if (isListening.value) {
      return;
    }

    try {
      await updateWindowInfo();
      await invoke("start_mouse_listener");

      unlistenFn.value = await listen("mouse-event", (event) => {
        if (event.payload.event_type === "Move") {
          handleMouseMove(event);
        }
      });

      isListening.value = true;
    } catch (error) {
      console.error("启动窗口穿透监听失败:", error);
    }
  }

  /**
   * 停止监听
   */
  async function stopListening() {
    if (!isListening.value) {
      return;
    }

    try {
      await invoke("stop_mouse_listener");

      if (unlistenFn.value) {
        unlistenFn.value();
        unlistenFn.value = null;
      }

      isListening.value = false;
    } catch (error) {
      console.error("停止窗口穿透监听失败:", error);
    }
  }

  /**
   * 重新扫描元素（当 DOM 变化时调用）
   */
  function rescanElements() {
    scanThroughElements();
  }

  /**
   * 注册并启动监听
   */
  async function register() {
    await nextTick();
    await updateWindowInfo();
    scanThroughElements();
    await startListening();
    
    // 监听 DOM 变化，自动重新扫描元素
    setupMutationObserver();
  }

  /**
   * 设置 MutationObserver 监听 DOM 变化
   */
  function setupMutationObserver() {
    // 如果已经有观察器，先停止
    if (mutationObserver.value) {
      stopMutationObserver();
    }

    // 创建新的观察器
    mutationObserver.value = new MutationObserver((mutations) => {
      let needsRescan = false;
      
      mutations.forEach((mutation) => {
        // 检查是否有属性变化
        if (mutation.type === 'attributes' && mutation.attributeName === 'through-listener') {
          needsRescan = true;
        }
        
        // 检查是否有子节点变化（新增/删除元素）
        if (mutation.type === 'childList') {
          // 检查新增的节点
          mutation.addedNodes.forEach((node) => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              if (node.hasAttribute('through-listener') || 
                  node.querySelector('[through-listener]')) {
                needsRescan = true;
              }
            }
          });
          
          // 检查删除的节点
          mutation.removedNodes.forEach((node) => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              if (node.hasAttribute('through-listener') || 
                  node.querySelector('[through-listener]')) {
                needsRescan = true;
              }
            }
          });
        }
      });

      if (needsRescan) {
        scanThroughElements();
      }
    });

    mutationObserver.value.observe(document.body, {
      attributes: true,
      attributeFilter: ['through-listener'],
      childList: true,
      subtree: true,
    });
  }

  /**
   * 停止 MutationObserver
   */
  function stopMutationObserver() {
    if (mutationObserver.value) {
      mutationObserver.value.disconnect();
      mutationObserver.value = null;
    }
  }

  /**
   * 注销监听
   */
  async function unregister() {
    await stopListening();
    stopMutationObserver();
    registeredElements.value.clear();
    // 确保窗口重置为非穿透状态，可以接收点击
    await setWindowIgnore(false);
  }

  return {
    isListening,
    registeredElements,
    register,
    unregister,
    rescanElements,
    setWindowIgnore,
    windowPosition,
    scaleFactor,
  };
}

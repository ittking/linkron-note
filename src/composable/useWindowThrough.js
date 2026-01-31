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
  const isMacOS = ref(false);

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

      console.log('[WindowThrough] updateWindowInfo - 窗口位置:', { x: pos.x, y: pos.y });
      console.log('[WindowThrough] updateWindowInfo - 缩放比例:', scale);
      console.log('[WindowThrough] updateWindowInfo - 屏幕信息:', {
        screenX: window.screenX,
        screenY: window.screenY,
        innerWidth: window.innerWidth,
        innerHeight: window.innerHeight
      });

      windowPosition.value = { x: pos.x, y: pos.y };
      scaleFactor.value = scale;
    } catch (error) {
      console.error('[WindowThrough] 获取窗口信息失败:', error);
    }
  }

  /**
   * 设置窗口穿透状态
   * @param {boolean} ignore - true: 忽略鼠标（穿透），false: 不忽略鼠标（不穿透）
   */
  async function setWindowIgnore(ignore) {
    try {
      await initWindow();
      console.log('[WindowThrough] setWindowIgnore - 设置状态:', {
        ignore,
        meaning: ignore ? '穿透（鼠标事件穿透到下方窗口）' : '不穿透（窗口接收鼠标事件）'
      });
      await currentWindow.value.setIgnoreCursorEvents(ignore);
    } catch (error) {
      console.error('[WindowThrough] 设置窗口穿透失败:', error);
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

    console.log('[WindowThrough] getElementScreenBounds - 元素:', element.tagName, element.id || element.className);
    console.log('[WindowThrough] getElementScreenBounds - getBoundingClientRect:', {
      left: rect.left,
      top: rect.top,
      width: rect.width,
      height: rect.height,
      right: rect.right,
      bottom: rect.bottom
    });
    console.log('[WindowThrough] getElementScreenBounds - 计算结果:', {
      winX, winY, scale,
      screenX, screenY,
      screenWidth, screenHeight,
      right: screenX + screenWidth,
      bottom: screenY + screenHeight
    });

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
    const inBounds = (
      mouseX >= bounds.x &&
      mouseX <= bounds.right &&
      mouseY >= bounds.y &&
      mouseY <= bounds.bottom
    );

    console.log('[WindowThrough] isMouseInBounds - 判断结果:', {
      mouseX, mouseY,
      bounds,
      conditions: {
        xCheck: mouseX >= bounds.x,
        rightCheck: mouseX <= bounds.right,
        yCheck: mouseY >= bounds.y,
        bottomCheck: mouseY <= bounds.bottom
      },
      inBounds
    });

    return inBounds;
  }

  /**
   * 扫描页面中所有带有 through-listener 属性且值为 true 的元素
   */
  function scanThroughElements() {
    const elements = document.querySelectorAll("[through-listener]");
    console.log('[WindowThrough] scanThroughElements - 找到的所有带 through-listener 属性的元素:', elements.length);

    registeredElements.value.clear();

    elements.forEach((element) => {
      // 只处理 through-listener 属性值为 true 的元素
      const listenerValue = element.getAttribute("through-listener");
      console.log('[WindowThrough] scanThroughElements - 元素属性值:', {
        tag: element.tagName,
        id: element.id,
        className: element.className,
        listenerValue
      });

      if (listenerValue !== "true" && listenerValue !== true) {
        return;
      }

      const id = element.id || `through-${Date.now()}-${Math.random()}`;
      const bounds = getElementScreenBounds(element);

      console.log('[WindowThrough] scanThroughElements - 注册元素:', {
        id,
        tag: element.tagName,
        className: element.className
      });

      registeredElements.value.set(id, {
        element,
        bounds,
        id,
      });
    });

    console.log('[WindowThrough] scanThroughElements - 最终注册的元素数量:', registeredElements.value.size);
  }

  /**
   * 处理鼠标移动事件
   * @param {Object} event - 鼠标事件对象
   */
  async function handleMouseMove(event) {
    const { x, y } = event.payload;

    console.log('[WindowThrough] handleMouseMove - 鼠标位置:', { x, y });
    console.log('[WindowThrough] handleMouseMove - 已注册元素数量:', registeredElements.value.size);

    // 实时更新窗口位置（处理窗口拖拽后的位置变化）
    await updateWindowInfo();

    // 重新计算所有元素的屏幕边界
    for (const [id, data] of registeredElements.value) {
      data.bounds = getElementScreenBounds(data.element);
    }

    // 检查鼠标是否在任何注册的元素范围内
    let inAnyElement = false;
    let matchedElementId = null;
    for (const [id, data] of registeredElements.value) {
      console.log('[WindowThrough] handleMouseMove - 检查元素:', id, {
        mouseX: x, mouseY: y,
        bounds: data.bounds,
        inBounds: isMouseInBounds(x, y, data.bounds)
      });

      if (isMouseInBounds(x, y, data.bounds)) {
        inAnyElement = true;
        matchedElementId = id;
        break;
      }
    }

    console.log('[WindowThrough] handleMouseMove - 检测结果:', {
      inAnyElement,
      matchedElementId,
      willSetIgnore: !inAnyElement
    });

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
      // 只有在非 macOS 平台才使用 rdev 库监听鼠标事件
      // 在 macOS 平台上，我们使用 Tauri 的内置事件系统
      if (!isMacOS.value) {
        await invoke("start_mouse_listener");

        unlistenFn.value = await listen("mouse-event", (event) => {
          if (event.payload.event_type === "Move") {
            handleMouseMove(event);
          }
        });
      } else {
        // 在 macOS 平台上，使用 Tauri 的内置事件系统
        console.log('[WindowThrough] 启动 macOS 特定的鼠标事件监听');
        // 添加窗口鼠标移动事件监听
        const window = getCurrentWindow();
        unlistenFn.value = await window.onMouseMove((event) => {
          handleMouseMoveMacOS(event);
        });
      }

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
      if (!isMacOS.value) {
        await invoke("stop_mouse_listener");

        if (unlistenFn.value) {
          unlistenFn.value();
          unlistenFn.value = null;
        }
      } else {
        // 在 macOS 平台上，停止 Tauri 的内置事件监听
        if (unlistenFn.value) {
          unlistenFn.value();
          unlistenFn.value = null;
        }
      }

      isListening.value = false;
    } catch (error) {
      console.error("停止窗口穿透监听失败:", error);
    }
  }

  /**
   * 处理 macOS 平台上的鼠标移动事件
   * @param {Object} event - 鼠标事件对象
   */
  async function handleMouseMoveMacOS(event) {
    const { x, y } = event;

    console.log('[WindowThrough] handleMouseMoveMacOS - 鼠标位置:', { x, y });
    console.log('[WindowThrough] handleMouseMoveMacOS - 已注册元素数量:', registeredElements.value.size);

    // 实时更新窗口位置（处理窗口拖拽后的位置变化）
    await updateWindowInfo();

    // 重新计算所有元素的屏幕边界
    for (const [id, data] of registeredElements.value) {
      data.bounds = getElementScreenBounds(data.element);
    }

    // 检查鼠标是否在任何注册的元素范围内
    let inAnyElement = false;
    let matchedElementId = null;
    for (const [id, data] of registeredElements.value) {
      console.log('[WindowThrough] handleMouseMoveMacOS - 检查元素:', id, {
        mouseX: x, mouseY: y,
        bounds: data.bounds,
        inBounds: isMouseInBounds(x, y, data.bounds)
      });

      if (isMouseInBounds(x, y, data.bounds)) {
        inAnyElement = true;
        matchedElementId = id;
        break;
      }
    }

    console.log('[WindowThrough] handleMouseMoveMacOS - 检测结果:', {
      inAnyElement,
      matchedElementId,
      willSetIgnore: !inAnyElement
    });

    // 设置窗口穿透状态
    // 在元素范围内：不穿透（可以交互）
    // 不在元素范围内：穿透（点击穿透到下方窗口）
    await setWindowIgnore(!inAnyElement);
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

  // 初始化时检测操作系统
  async function init() {
    try {
      const os = await invoke('get_os');
      isMacOS.value = os === 'macos';
      console.log('[WindowThrough] 检测到操作系统:', os);
    } catch (error) {
      console.error('[WindowThrough] 检测操作系统失败:', error);
      isMacOS.value = false;
    }
  }

  // 初始化
  init();

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

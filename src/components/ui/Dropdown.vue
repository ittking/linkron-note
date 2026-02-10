<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'

const props = defineProps({
  position: {
    type: String,
    default: 'bottom',
    validator: (value) => ['bottom', 'bottom-end', 'bottom-start', 'top', 'top-end', 'top-start'].includes(value)
  },
  offset: {
    type: Number,
    default: 4
  },
  disabled: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['open', 'close'])

const isOpen = ref(false)
const triggerRef = ref(null)
const menuRef = ref(null)
const menuPosition = ref({ top: 0, left: 0, width: 0 })

const menuStyle = computed(() => ({
  top: `${menuPosition.value.top}px`,
  left: `${menuPosition.value.left}px`,
  width: menuPosition.value.width ? `${menuPosition.value.width}px` : 'auto',
  zIndex: 9999
}))

const placementClass = computed(() => {
  switch (props.position) {
    case 'bottom-end':
      return 'right-0'
    case 'bottom-start':
      return 'left-0'
    case 'top':
      return 'bottom-full'
    case 'top-end':
      return 'bottom-full right-0'
    case 'top-start':
      return 'bottom-full left-0'
    default: // bottom
      return 'left-0'
  }
})

async function toggle() {
  if (!props.disabled) {
    isOpen.value = !isOpen.value
    if (isOpen.value) {
      emit('open')
      await nextTick()
      updateMenuPosition()
    } else {
      emit('close')
    }
  }
}

function open() {
  if (!props.disabled && !isOpen.value) {
    isOpen.value = true
    emit('open')
    nextTick(() => {
      updateMenuPosition()
    })
  }
}

function close() {
  if (isOpen.value) {
    isOpen.value = false
    emit('close')
  }
}

function updateMenuPosition() {
  if (triggerRef.value) {
    const rect = triggerRef.value.getBoundingClientRect()
    const viewportWidth = window.innerWidth
    const viewportHeight = window.innerHeight
    
    let top = rect.bottom + props.offset
    let left = rect.left

    // 等待菜单内容渲染完成后获取实际宽度
    nextTick(() => {
      if (menuRef.value) {
        const menuWidth = menuRef.value.offsetWidth
        
        // 根据位置属性确定初始对齐方式
        if (props.position === 'bottom-end' || props.position === 'top-end') {
          // 右对齐：菜单右侧与触发器右侧对齐
          left = rect.right - menuWidth
        }

        // 检测并修正垂直位置
        const menuHeight = menuRef.value.offsetHeight
        if (top + menuHeight > viewportHeight - 16) {
          // 下方空间不足，改为向上显示
          top = rect.top - menuHeight - props.offset
        }

        // 检测并修正水平位置
        if (left < 16) {
          // 左侧超出，改为左对齐
          left = 16
        } else if (left + menuWidth > viewportWidth - 16) {
          // 右侧超出，改为右对齐
          left = viewportWidth - menuWidth - 16
        }

        menuPosition.value = {
          top,
          left,
          width: 'auto' // 让宽度跟随内容
        }
      }
    })
  }
}

function handleClickOutside(event) {
  if (triggerRef.value && !triggerRef.value.contains(event.target)) {
    close()
  }
}

// 暴露方法供外部调用
defineExpose({
  open,
  close,
  toggle
})

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('click', handleClickOutside)
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('click', handleClickOutside)
  }
})
</script>

<template>
  <div ref="triggerRef" class="relative inline-block">
    <!-- 触发器插槽 -->
    <div @click="toggle">
      <slot name="trigger" :isOpen="isOpen" :toggle="toggle" />
    </div>

    <!-- 下拉菜单 -->
    <Teleport to="body">
      <Transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0 scale-95"
        enter-to-class="opacity-100 scale-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-95"
      >
        <div
          v-if="isOpen"
          ref="menuRef"
          :class="[
            'fixed rounded-lg border shadow-lg overflow-hidden',
            'bg-base-100 border-base-300',
            'transform origin-top-left'
          ]"
          :style="menuStyle"
        >
          <slot :close="close" :isOpen="isOpen" />
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
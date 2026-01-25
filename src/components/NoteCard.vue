<script setup>
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { MoreHorizontal, ExternalLink, Edit, Trash2, ChevronDown, ChevronUp } from 'lucide-vue-next'
import dayjs from 'dayjs'
import 'dayjs/locale/zh-cn'

dayjs.locale('zh-cn')

const props = defineProps({
  note: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['click', 'open', 'edit', 'delete'])

const menuVisible = ref(false)
const isExpanded = ref(false)
const contentRef = ref(null)
const isOverflowing = ref(false)
const MAX_HEIGHT = 120 // 最大高度，超过这个高度显示展开按钮

// 格式化日期 - 精确到秒，不包含星期
const formattedDate = computed(() => {
  const date = dayjs(props.note.createdAt)
  return date.format('YYYY-MM-DD HH:mm:ss')
})

// 检查内容是否溢出
function checkOverflow() {
  if (contentRef.value) {
    const contentText = props.note.content || ''
    // 预估行高，判断是否需要展开
    const lineHeight = 22 // 假设每行约22px
    const estimatedLines = Math.ceil(contentText.length / 40) // 假设每行约40个字符
    const estimatedHeight = estimatedLines * lineHeight
    isOverflowing.value = estimatedHeight > MAX_HEIGHT && contentText.length > 80
  }
}

// 切换展开/收起
function toggleExpand(event) {
  event.stopPropagation()
  isExpanded.value = !isExpanded.value
}

// 菜单项点击处理
function handleMenuClick(action) {
  menuVisible.value = false
  if (action === 'open') {
    emit('open', props.note)
  } else if (action === 'edit') {
    emit('edit', props.note)
  } else if (action === 'delete') {
    emit('delete', props.note)
  }
}

// 卡片点击
function handleCardClick() {
  emit('click', props.note)
}

// 点击外部关闭菜单
function handleClickOutside(event) {
  if (menuVisible.value) {
    menuVisible.value = false
  }
}

// 生命周期钩子
onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  checkOverflow()
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div
    class="note-card bg-base-100 border border-base-200 rounded-lg p-4 mb-3 cursor-pointer transition-all duration-200 hover:shadow-md"
    @click="handleCardClick"
  >
    <!-- 顶部：日期 + 菜单 -->
    <div class="flex items-center justify-between mb-3">
      <span class="text-xs text-base-content/50">{{ formattedDate }}</span>
      <div class="relative">
        <button
          @click.stop="menuVisible = !menuVisible"
          class="w-6 h-6 rounded flex items-center justify-center text-base-content/40 hover:text-base-content hover:bg-base-200 transition-colors"
        >
          <MoreHorizontal :size="20" />
        </button>
        
        <!-- 下拉菜单 -->
        <div
          v-if="menuVisible"
          class="absolute right-0 top-8 z-10 bg-base-100 border border-base-200 rounded-lg shadow-xl min-w-[120px] py-1"
          @click.stop
        >
          <button
            v-if="note.sourceUrl"
            @click="handleMenuClick('open')"
            class="w-full px-3 py-2 text-left text-xs text-base-content hover:bg-base-200 flex items-center gap-2 transition-colors"
          >
            <ExternalLink :size="14" />
            打开链接
          </button>
          <button
            @click="handleMenuClick('edit')"
            class="w-full px-3 py-2 text-left text-xs text-base-content hover:bg-base-200 flex items-center gap-2 transition-colors"
          >
            <Edit :size="14" />
            编辑
          </button>
          <button
            @click="handleMenuClick('delete')"
            class="w-full px-3 py-2 text-left text-xs text-error hover:bg-base-200 flex items-center gap-2 transition-colors"
          >
            <Trash2 :size="14" />
            删除
          </button>
        </div>
      </div>
    </div>

    <!-- 内容 -->
    <div v-if="note.content" class="mb-3">
      <div
        ref="contentRef"
        class="text-sm text-base-content leading-relaxed whitespace-pre-wrap break-words"
        :class="{
          'line-clamp-5': !isExpanded && isOverflowing,
          'max-h-[120px] overflow-hidden': !isExpanded && isOverflowing
        }"
      >
        {{ note.content }}
      </div>
      
      <!-- 展开/收起按钮 -->
      <button
        v-if="isOverflowing"
        @click="toggleExpand"
        class="mt-2 text-xs text-primary hover:text-primary/80 flex items-center gap-1 transition-colors"
      >
        <template v-if="!isExpanded">
          展开全文
          <ChevronDown :size="14" />
        </template>
        <template v-else>
          收起
          <ChevronUp :size="14" />
        </template>
      </button>
    </div>

    <!-- 图片列表 -->
    <div v-if="note.images && note.images.length > 0" class="flex flex-wrap gap-2">
      <img
        v-for="(img, index) in note.images"
        :key="index"
        :src="img"
        class="w-20 h-20 rounded-lg object-cover border border-base-200"
        alt="Note image"
      />
    </div>
  </div>
</template>
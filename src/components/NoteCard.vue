<script setup>
import { computed, ref } from 'vue'
import { MoreVertical, ExternalLink, Edit, Trash2 } from 'lucide-vue-next'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import 'dayjs/locale/zh-cn'

dayjs.extend(relativeTime)
dayjs.locale('zh-cn')

const props = defineProps({
  note: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['click', 'open', 'edit', 'delete'])

const menuVisible = ref(false)

// 格式化日期
const formattedDate = computed(() => {
  return dayjs(props.note.createdAt).format('YYYY-MM-DD HH:mm')
})

// 相对时间
const relativeTimeText = computed(() => {
  const now = dayjs()
  const noteTime = dayjs(props.note.createdAt)
  const diffHours = now.diff(noteTime, 'hour')
  
  if (diffHours < 1) {
    return '刚刚'
  } else if (diffHours < 24) {
    return noteTime.fromNow()
  } else if (diffHours < 24 * 7) {
    return noteTime.fromNow()
  } else {
    return formattedDate.value
  }
})

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
</script>

<template>
  <div
    class="note-card bg-base-100 border border-base-200 rounded-lg p-4 mb-3 cursor-pointer transition-all duration-200 hover:shadow-md"
    @click="handleCardClick"
  >
    <!-- 顶部：日期 + 菜单 -->
    <div class="flex items-center justify-between mb-3">
      <span class="text-xs text-base-content/50">{{ relativeTimeText }}</span>
      <div class="relative">
        <button
          @click.stop="menuVisible = !menuVisible"
          class="w-6 h-6 rounded flex items-center justify-center text-base-content/40 hover:text-base-content hover:bg-base-200 transition-colors"
        >
          <MoreVertical :size="16" />
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
    <div v-if="note.content" class="text-sm text-base-content leading-relaxed mb-3 whitespace-pre-wrap">
      {{ note.content }}
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
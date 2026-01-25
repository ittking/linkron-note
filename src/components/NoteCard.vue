<script setup>
import { computed, ref } from 'vue'
import { MoreVertical, ExternalLink, Edit, Trash2 } from 'lucide-vue-next'

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
  const date = new Date(props.note.createdAt)
  const now = new Date()
  const diff = now - date

  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return Math.floor(diff / 60000) + '分钟前'
  if (diff < 86400000) return Math.floor(diff / 3600000) + '小时前'
  if (diff < 604800000) return Math.floor(diff / 86400000) + '天前'

  return date.toLocaleDateString('zh-CN')
})

// 类型标签样式
const typeConfig = {
  link: { label: '链接', color: 'text-[#60a5fa]' },
  image: { label: '图片', color: 'text-[#fbbf24]' },
  text: { label: '文字', color: 'text-[#a78bfa]' }
}

const typeInfo = computed(() => {
  return typeConfig[props.note.type] || typeConfig.text
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
    class="note-card bg-[#141417] border border-[#2a2a32] rounded-xl p-3.5 mb-2.5 cursor-pointer transition-all duration-200 relative overflow-hidden hover:border-[#3a3a45] hover:-translate-y-0.5 hover:shadow-lg"
    @click="handleCardClick"
  >
    <!-- 顶部渐变条 -->
    <div class="absolute top-0 left-0 right-0 h-0.5 bg-gradient-to-r from-[#00ff88] to-transparent opacity-0 transition-opacity duration-200 note-card:hover:opacity-100"></div>

    <!-- 头部：标题 + 菜单 -->
    <div class="flex items-center justify-between mb-2">
      <div class="text-sm font-medium text-[#e8e8ed] leading-relaxed line-clamp-2 flex-1 mr-2">
        {{ note.title }}
      </div>
      <div class="relative">
        <button
          @click.stop="menuVisible = !menuVisible"
          class="w-6 h-6 rounded flex items-center justify-center text-[#4a4a55] hover:text-[#e8e8ed] hover:bg-[#2a2a32] transition-colors"
        >
          <MoreVertical :size="16" />
        </button>
        
        <!-- 下拉菜单 -->
        <div
          v-if="menuVisible"
          class="absolute right-0 top-8 z-10 bg-[#1a1a1f] border border-[#2a2a32] rounded-lg shadow-xl min-w-[140px] py-1"
          @click.stop
        >
          <button
            @click="handleMenuClick('open')"
            class="w-full px-3 py-2 text-left text-sm text-[#e8e8ed] hover:bg-[#2a2a32] flex items-center gap-2 transition-colors"
          >
            <ExternalLink :size="14" />
            新窗口打开
          </button>
          <button
            @click="handleMenuClick('edit')"
            class="w-full px-3 py-2 text-left text-sm text-[#e8e8ed] hover:bg-[#2a2a32] flex items-center gap-2 transition-colors"
          >
            <Edit :size="14" />
            编辑
          </button>
          <button
            @click="handleMenuClick('delete')"
            class="w-full px-3 py-2 text-left text-sm text-[#ef4444] hover:bg-[#2a2a32] flex items-center gap-2 transition-colors"
          >
            <Trash2 :size="14" />
            删除
          </button>
        </div>
      </div>
    </div>

    <!-- 内容预览 -->
    <div v-if="note.content" class="text-xs text-[#6b6b76] leading-relaxed line-clamp-2 mb-2">
      {{ note.content }}
    </div>

    <!-- 图片缩略图 -->
    <div v-if="note.images && note.images.length > 0" class="flex gap-2 mt-2.5 overflow-x-auto pb-1">
      <img
        v-for="(img, index) in note.images.slice(0, 3)"
        :key="index"
        :src="img"
        class="w-[60px] h-[60px] rounded-lg object-cover flex-shrink-0 border border-[#2a2a32]"
        alt="Note image"
      />
    </div>

    <!-- 底部：类型标签 + 日期 -->
    <div class="flex items-center justify-between mt-3 pt-3 border-t border-[#2a2a32]">
      <span :class="['text-xs font-medium', typeInfo.color]">
        #{{ typeInfo.label }}
      </span>
      <span class="text-[11px] text-[#4a4a55] font-mono">
        {{ formattedDate }}
      </span>
    </div>
  </div>
</template>
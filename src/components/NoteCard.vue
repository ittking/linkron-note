<script setup>
import { computed } from 'vue'

const props = defineProps({
  note: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['click', 'open', 'edit', 'delete'])

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
function handleMenuClick({ key }) {
  if (key === 'open') {
    emit('open', props.note)
  } else if (key === 'edit') {
    emit('edit', props.note)
  } else if (key === 'delete') {
    emit('delete', props.note)
  }
}

// 卡片点击
function handleCardClick() {
  emit('click', props.note)
}
</script>

<template>
  <a-card
    class="note-card"
    :bordered="false"
    :body-style="{ padding: '14px' }"
    @click="handleCardClick"
  >
    <!-- 头部：标题 + 菜单 -->
    <div class="note-card-header">
      <div class="note-card-title">{{ note.title }}</div>
      <a-dropdown trigger="click" @click.stop>
        <a-button type="text" size="small" class="note-card-menu">
          <template #icon>
            <span class="text-lg">⋮</span>
          </template>
        </a-button>
        <template #overlay>
          <a-menu @click="handleMenuClick">
            <a-menu-item key="open">
              <span>🔗</span>
              <span class="ml-2">新窗口打开</span>
            </a-menu-item>
            <a-menu-item key="edit">
              <span>✏️</span>
              <span class="ml-2">编辑</span>
            </a-menu-item>
            <a-menu-item key="delete" danger>
              <span>🗑️</span>
              <span class="ml-2">删除</span>
            </a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>
    </div>

    <!-- 内容预览 -->
    <div v-if="note.content" class="note-card-preview">
      {{ note.content }}
    </div>

    <!-- 图片缩略图 -->
    <div v-if="note.images && note.images.length > 0" class="note-card-images">
      <img
        v-for="(img, index) in note.images.slice(0, 3)"
        :key="index"
        :src="img"
        class="note-card-image"
        alt="Note image"
      />
    </div>

    <!-- 底部：类型标签 + 日期 -->
    <div class="note-card-footer">
      <span :class="['note-card-type', typeInfo.color]">
        #{{ typeInfo.label }}
      </span>
      <span class="note-card-date">{{ formattedDate }}</span>
    </div>
  </a-card>
</template>

<style scoped>
.note-card {
  background: #141417;
  border: 1px solid #2a2a32;
  border-radius: 10px;
  margin-bottom: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
  overflow: hidden;
}

.note-card:hover {
  border-color: #3a3a45;
  transform: translateY(-2px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.note-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, #00ff88, transparent);
  opacity: 0;
  transition: opacity 0.2s ease;
}

.note-card:hover::before {
  opacity: 1;
}

.note-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.note-card-title {
  font-size: 14px;
  font-weight: 500;
  color: #e8e8ed;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  flex: 1;
  margin-right: 8px;
}

.note-card-menu {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  color: #4a4a55;
  padding: 0;
}

.note-card-menu:hover {
  color: #e8e8ed;
}

.note-card-preview {
  font-size: 12px;
  color: #6b6b76;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.note-card-images {
  display: flex;
  gap: 8px;
  margin-top: 10px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.note-card-image {
  width: 60px;
  height: 60px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
  border: 1px solid #2a2a32;
}

.note-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #2a2a32;
}

.note-card-type {
  font-size: 12px;
  font-weight: 500;
}

.note-card-date {
  font-size: 11px;
  color: #4a4a55;
  font-family: 'JetBrains Mono', 'Courier New', monospace;
}
</style>
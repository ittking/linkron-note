<template>
  <div class="terminal-tabs">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="tab"
      :class="{ active: tab.id === activeTabId }"
      @click="selectTab(tab.id)"
    >
      <span class="tab-title">{{ tab.title }}</span>
      <span
        class="tab-close"
        @click.stop="closeTab(tab.id)"
        v-if="tabs.length > 1"
      >×</span>
    </div>
    <div class="tab-add" @click="addTab">+</div>
  </div>
</template>

<script setup>
const props = defineProps({
  tabs: { type: Array, required: true },
  activeTabId: { type: [String, null], default: null }
})

const emit = defineEmits(['select', 'add', 'close'])

const selectTab = (id) => {
  console.log('TerminalTab: selectTab clicked, id:', id)
  emit('select', id)
}
const addTab = () => emit('add')
const closeTab = (id) => emit('close', id)
</script>

<style scoped>
.terminal-tabs {
  display: flex;
  background: #2d2d2d;
  border-bottom: 1px solid #3e3e3e;
  overflow-x: auto;
}

.tab {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  border-right: 1px solid #3e3e3e;
  user-select: none;
  white-space: nowrap;
  background: #2d2d2d;
  transition: background 0.2s;
}

.tab:hover {
  background: #383838;
}

.tab.active {
  background: #1e1e1e;
}

.tab-title {
  font-size: 13px;
  color: #d4d4d4;
}

.tab-close {
  margin-left: 8px;
  opacity: 0.6;
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  padding: 0 4px;
}

.tab-close:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.tab-add {
  padding: 8px 16px;
  cursor: pointer;
  opacity: 0.6;
  font-size: 18px;
  line-height: 1;
  transition: opacity 0.2s;
}

.tab-add:hover {
  opacity: 1;
}
</style>
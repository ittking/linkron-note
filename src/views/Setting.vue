<script setup>
import { ref } from 'vue'
import { Settings, Bot, Palette } from 'lucide-vue-next'
import PreferencesSetting from '../components/PreferencesSetting.vue'
import ModelSetting from '../components/ModelSetting.vue'
import ThemeSetting from '../components/ThemeSetting.vue'

const activeTab = ref('preferences')

const tabs = [
  { id: 'preferences', label: '偏好', icon: Settings },
  { id: 'model', label: '模型', icon: Bot },
  { id: 'theme', label: '主题', icon: Palette }
]

function setActiveTab(tabId) {
  activeTab.value = tabId
}
</script>

<template>
  <div class="h-full flex flex-col max-w-200 mx-auto">
    <!-- Tab 导航 -->
    <div role="tablist" class="tabs tabs-boxed px-4 pt-4">
      <a v-for="tab in tabs" :key="tab.id" role="tab" @click="setActiveTab(tab.id)" :class="[
        'tab gap-2',
        activeTab === tab.id ? 'tab-active' : ''
      ]">
        <component :is="tab.icon" :size="14" />
        <span>{{ tab.label }}</span>
      </a>
    </div>

    <!-- Tab 内容 -->
    <div class="flex-1 p-4 pt-2 overflow-y-auto no-scrollbar w-full">
      <PreferencesSetting v-if="activeTab === 'preferences'" />
      <ModelSetting v-if="activeTab === 'model'" />
      <ThemeSetting v-if="activeTab === 'theme'" />
    </div>
  </div>
</template>
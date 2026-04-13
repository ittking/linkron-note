<script setup>
import { ref } from 'vue'
import { User, Settings, Cloud, Palette, Info } from 'lucide-vue-next'
import AccountSetting from '../components/AccountSetting.vue'
import PreferencesSetting from '../components/PreferencesSetting.vue'
import SyncSetting from '../components/SyncSetting.vue'
import ThemeSetting from '../components/ThemeSetting.vue'
import AboutSetting from '../components/AboutSetting.vue'

const activeTab = ref('account')

const tabs = [
  { id: 'account', label: '账户', icon: User },
  { id: 'preferences', label: '偏好', icon: Settings },
  { id: 'sync', label: '同步', icon: Cloud },
  { id: 'theme', label: '主题', icon: Palette },
  { id: 'about', label: '关于', icon: Info }
]

function setActiveTab(tabId) {
  activeTab.value = tabId
}
</script>

<template>
  <div class="h-full flex flex-col max-w-200 mx-auto pb-2">
    <!-- Tab 导航 -->
    <div role="tablist" class="px-1 pt-4 pb-2  gap-1">
      <a v-for="tab in tabs" :key="tab.id" role="tab" @click="setActiveTab(tab.id)" :class="[
        'tab gap-1.5 text-xs',
        activeTab === tab.id ? 'text-primary' : ''
      ]">
        <component :is="tab.icon" :size="12" />
        <span>{{ tab.label }}</span>
      </a>
    </div>

    <!-- Tab 内容 -->
    <div class="flex-1 p-4 pt-2 overflow-y-auto no-scrollbar w-full">
      <AccountSetting v-if="activeTab === 'account'" />
      <PreferencesSetting v-if="activeTab === 'preferences'" />
      <SyncSetting v-if="activeTab === 'sync'" />
      <ThemeSetting v-if="activeTab === 'theme'" />
      <AboutSetting v-if="activeTab === 'about'" />
    </div>
  </div>
</template>
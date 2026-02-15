<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from '../store/settingStore'
import { Bot, Plus, MessageSquare } from 'lucide-vue-next'
import Button from './ui/Button.vue'
import ProviderCard from './ProviderCard.vue'
import AddProviderDialog from './AddProviderDialog.vue'
import SelectModelDialog from './SelectModelDialog.vue'
import PromptCard from './PromptCard.vue'
import PromptDialog from './PromptDialog.vue'

const settingStore = useSettingStore()

const providers = ref([])
const selectedProviderId = ref(null)

const showAddDialog = ref(false)
const showSelectDialog = ref(false)
const currentProvider = ref(null)

// 提示词相关
const prompts = ref([])
const showPromptDialog = ref(false)
const currentPrompt = ref(null)

onMounted(async () => {
  await loadProviders()
  await loadPrompts()
})

async function loadProviders() {
  try {
    const data = await settingStore.get('model.providers', [])
    providers.value = data || []
    
    const activeId = await settingStore.get('model.activeProviderId', null)
    if (activeId) {
      selectedProviderId.value = activeId
    }
  } catch (error) {
    console.error('Failed to load providers:', error)
  }
}

async function saveProviders() {
  try {
    await settingStore.set('model.providers', providers.value)
    if (selectedProviderId.value) {
      await settingStore.set('model.activeProviderId', selectedProviderId.value)
    }
  } catch (error) {
    console.error('Failed to save providers:', error)
  }
}

async function handleAddProvider(data) {
  const newProvider = {
    id: `provider-${Date.now()}`,
    provider: data.provider,
    customName: data.customName || '',
    apiKey: data.apiKey,
    apiUrl: data.apiUrl || '',
    models: [],
    currentModel: null,
    createdAt: new Date().toISOString()
  }

  providers.value.push(newProvider)
  selectedProviderId.value = newProvider.id
  
  await saveProviders()
  showAddDialog.value = false
}

async function handleSelectProvider(provider) {
  selectedProviderId.value = provider.id
  await saveProviders()
}

async function handleDeleteProvider(provider) {
  if (!confirm(`确定要删除供应商 "${provider.customName || provider.provider}" 吗？`)) {
    return
  }

  const index = providers.value.findIndex(p => p.id === provider.id)
  if (index > -1) {
    providers.value.splice(index, 1)
    
    if (selectedProviderId.value === provider.id) {
      selectedProviderId.value = providers.value.length > 0 ? providers.value[0].id : null
    }
    
    await saveProviders()
  }
}

function handleChooseModel(provider) {
  currentProvider.value = provider
  showSelectDialog.value = true
}

async function handleLoadModels(provider) {
  try {
    const result = await invoke('load_provider_models', {
      provider: provider.provider,
      apiKey: provider.apiKey,
      apiUrl: provider.apiUrl
    })

    const index = providers.value.findIndex(p => p.id === provider.id)
    if (index > -1) {
      providers.value[index].models = result || []
      await saveProviders()
    }
  } catch (error) {
    console.error('Failed to load models:', error)
    alert('加载模型列表失败: ' + error)
  }
}

async function handleSelectModel(model) {
  const index = providers.value.findIndex(p => p.id === currentProvider.value.id)
  if (index > -1) {
    providers.value[index].currentModel = model
    selectedProviderId.value = currentProvider.value.id
    await saveProviders()
  }
}

async function handleAddCustomModel(modelName) {
  const index = providers.value.findIndex(p => p.id === currentProvider.value.id)
  if (index > -1) {
    providers.value[index].currentModel = modelName
    selectedProviderId.value = currentProvider.value.id
    await saveProviders()
  }
}

async function handleLoadModelsInDialog() {
  if (currentProvider.value) {
    await handleLoadModels(currentProvider.value)
  }
}

const currentModels = computed(() => {
  if (!currentProvider.value) return []
  return currentProvider.value.models || []
})

// 提示词相关功能
async function loadPrompts() {
  try {
    const data = await settingStore.get('model.prompts', [])
    prompts.value = data || []
  } catch (error) {
    console.error('Failed to load prompts:', error)
  }
}

async function savePrompts() {
  try {
    await settingStore.set('model.prompts', prompts.value)
  } catch (error) {
    console.error('Failed to save prompts:', error)
  }
}

function handleAddPrompt() {
  currentPrompt.value = null
  showPromptDialog.value = true
}

function handleEditPrompt(prompt) {
  currentPrompt.value = prompt
  showPromptDialog.value = true
}

async function handleSavePrompt(data) {
  const index = prompts.value.findIndex(p => p.id === data.id)
  if (index > -1) {
    prompts.value[index] = data
  } else {
    prompts.value.push(data)
  }
  await savePrompts()
  showPromptDialog.value = false
}

async function handleDeletePrompt(prompt) {
  if (!confirm(`确定要删除提示词 "${prompt.name}" 吗？`)) {
    return
  }

  const index = prompts.value.findIndex(p => p.id === prompt.id)
  if (index > -1) {
    prompts.value.splice(index, 1)
    await savePrompts()
  }
}
</script>

<template>
  <div class="space-y-4">
    <!-- 模型供应商模块 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between mb-4">
          <h2 class="card-title text-sm font-medium flex items-center gap-2">
            <Bot :size="16" />
            模型供应商
          </h2>
          <Button variant="primary" size="sm" @click="showAddDialog = true">
            <Plus :size="14" />
            添加
          </Button>
        </div>

        <div v-if="providers.length === 0" class="text-center py-8 text-base-content/40">
          <Bot :size="40" class="mx-auto mb-3 opacity-50" />
          <p class="text-sm">暂无供应商配置</p>
          <p class="text-xs mt-1">点击右上角按钮添加供应商</p>
        </div>

        <div v-else class="space-y-3">
          <ProviderCard
            v-for="provider in providers"
            :key="provider.id"
            :provider="provider"
            :is-active="provider.id === selectedProviderId"
            @select="handleSelectProvider"
            @delete="handleDeleteProvider"
            @load-models="handleLoadModels"
            @choose-model="handleChooseModel"
          />
        </div>
      </div>
    </div>

    <!-- 提示词模块 -->
    <div class="card bg-base-200 shadow-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between mb-4">
          <h2 class="card-title text-sm font-medium flex items-center gap-2">
            <MessageSquare :size="16" />
            提示词
          </h2>
          <Button variant="primary" size="sm" @click="handleAddPrompt">
            <Plus :size="14" />
            添加
          </Button>
        </div>

        <div v-if="prompts.length === 0" class="text-center py-8 text-base-content/40">
          <MessageSquare :size="40" class="mx-auto mb-3 opacity-50" />
          <p class="text-sm">暂无提示词配置</p>
          <p class="text-xs mt-1">点击右上角按钮添加提示词</p>
        </div>

        <div v-else class="space-y-3">
          <PromptCard
            v-for="prompt in prompts"
            :key="prompt.id"
            :prompt="prompt"
            @delete="handleDeletePrompt"
            @edit="handleEditPrompt"
          />
        </div>
      </div>
    </div>
  </div>

  <AddProviderDialog
    v-model:show="showAddDialog"
    @save="handleAddProvider"
    @close="showAddDialog = false"
  />

  <SelectModelDialog
    v-model:show="showSelectDialog"
    :provider="currentProvider"
    :models="currentModels"
    @select="handleSelectModel"
    @load-models="handleLoadModelsInDialog"
    @add-custom="handleAddCustomModel"
    @close="showSelectDialog = false"
  />

  <PromptDialog
    v-model:show="showPromptDialog"
    :prompt="currentPrompt"
    @save="handleSavePrompt"
    @close="showPromptDialog = false"
  />
</template>
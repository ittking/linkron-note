<script setup>
import { ref, computed, watch } from 'vue'
import { Search, X, RefreshCw, Plus, Check, CheckCircle, XCircle, Loader2 } from 'lucide-vue-next'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'
import { useToast } from '../composables/useToast'
import { useAIChat } from '../composables/useAIChat'

const props = defineProps({
  show: {
    type: Boolean,
    default: false
  },
  provider: {
    type: Object,
    default: null
  },
  models: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['close', 'select', 'load-models', 'add-custom'])

const { showToast } = useToast()
const { testConnection } = useAIChat()
const searchQuery = ref('')
const customModel = ref('')
const tempSelectedModel = ref('')
const isTesting = ref(false)
const isLoadingModels = ref(false)

const filteredModels = computed(() => {
  if (!searchQuery.value) return props.models
  const query = searchQuery.value.toLowerCase()
  return props.models.filter(model => 
    model.toLowerCase().includes(query)
  )
})

function handleClose() {
  searchQuery.value = ''
  customModel.value = ''
  tempSelectedModel.value = ''
  emit('close')
}

function handleSelect(model) {
  tempSelectedModel.value = model
}

function handleConfirm() {
  if (tempSelectedModel.value) {
    emit('select', tempSelectedModel.value)
    handleClose()
  } else if (customModel.value.trim()) {
    handleAddCustom()
  } else {
    showToast('请先选择一个模型', 'error')
  }
}

async function handleLoadModels() {
  if (!props.provider) {
    showToast('请先配置模型供应商', 'error')
    return
  }

  isLoadingModels.value = true

  try {
    // 等待父组件加载模型列表
    // 使用 Promise 等待一小段时间让父组件处理
    emit('load-models', props.provider)

    // 等待父组件处理加载（通过监听 props.models 的变化）
    await new Promise(resolve => setTimeout(resolve, 500))
  } catch (error) {
    console.error('Failed to load models:', error)
    showToast('加载模型列表失败', 'error')
  } finally {
    isLoadingModels.value = false
  }
}

function handleAddCustom() {
  if (!customModel.value.trim()) {
    showToast('请输入模型名称', 'error')
    return
  }
  emit('add-custom', customModel.value.trim())
  customModel.value = ''
  handleClose()
}

async function handleTestModel() {
  const modelToTest = tempSelectedModel.value || customModel.value.trim()
  
  if (!modelToTest) {
    showToast('请先选择或输入模型名称', 'error')
    return
  }

  if (!props.provider) {
    showToast('请先配置模型供应商', 'error')
    return
  }

  if (!props.provider.apiKey) {
    showToast('请先配置 API Key', 'error')
    return
  }

  isTesting.value = true
  
  try {
    // 构造包含临时模型的 provider 对象
    const testProvider = {
      ...props.provider,
      currentModel: modelToTest
    }
    
    const result = await testConnection(testProvider)
    
    if (result.success) {
      showToast(result.message, 'success')
    } else {
      showToast(`模型连接失败: ${result.message}`, 'error')
    }
  } catch (error) {
    console.error('Model test error:', error)
    showToast(`模型连接失败: ${error.message}`, 'error')
  } finally {
    isTesting.value = false
  }
}

watch(() => props.show, (newVal) => {
  if (newVal) {
    searchQuery.value = ''
    customModel.value = ''
    // 回显之前选中的模型
    tempSelectedModel.value = props.provider?.currentModel || ''
    if (props.provider?.currentModel && !props.models.includes(props.provider.currentModel)) {
      customModel.value = props.provider.currentModel
    }
  } else {
    searchQuery.value = ''
    customModel.value = ''
    tempSelectedModel.value = ''
  }
})
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="card bg-base-100 shadow-xl w-full max-w-md mx-4 max-h-[85vh] flex flex-col">
      <div class="card-body p-5 flex flex-col">
        <div class="flex items-center justify-between mb-4 flex-shrink-0">
          <h3 class="card-title text-base font-medium">选择模型</h3>
          <button @click="handleClose" class="p-1 rounded hover:bg-base-200 transition-colors">
            <X :size="18" />
          </button>
        </div>

        <div class="space-y-3 flex-1 overflow-hidden flex flex-col p-2">
          <div class="flex gap-2 flex-shrink-0 items-center">
            <div class="flex-1 relative">
              <Search :size="14" class="absolute left-2 top-1/2 -translate-y-1/2 text-base-content/40" />
              <Input 
                v-model="searchQuery"
                placeholder="搜索模型..."
                size="sm"
                class="pl-7"
              />
            </div>
            <Button
              variant="ghost"
              size="sm"
              @click="handleLoadModels"
              :disabled="isLoadingModels"
            >
              <RefreshCw
                :size="14"
                :class="{ 'animate-spin': isLoadingModels }"
              />
            </Button>
          </div>

          <div class="flex-1 overflow-y-auto min-h-0 max-h-[300px] custom-scrollbar">
            <div v-if="filteredModels.length === 0" class="text-center py-6 text-base-content/40 text-sm">
              暂无模型，请先加载或添加自定义模型
            </div>
            <div v-else class="space-y-1">
              <div 
                v-for="model in filteredModels" 
                :key="model"
                @click="handleSelect(model)"
                :class="[
                  'p-2.5 rounded cursor-pointer transition-colors border',
                  tempSelectedModel === model
                    ? 'bg-primary/10 border-primary text-primary'
                    : 'bg-base-100 border-transparent hover:bg-base-200 hover:border-base-300'
                ]"
              >
                <div class="flex items-center justify-between">
                  <span class="text-sm">{{ model }}</span>
                  <Check 
                    v-if="tempSelectedModel === model" 
                    :size="14" 
                  />
                </div>
              </div>
            </div>
          </div>

          <div class="pt-3 border-t border-base-200 flex-shrink-0">
            <p class="text-xs text-base-content/60 mb-2">或添加自定义模型</p>
            <div class="flex gap-2 items-center">
              <Input 
                v-model="customModel"
                placeholder="输入模型名称，如: gpt-4"
                size="sm"
                class="flex-1"
                @keypress.enter="handleAddCustom"
              />
              <Button variant="primary" size="sm" @click="handleAddCustom">
                <Plus :size="14" />
              </Button>
            </div>
          </div>
        </div>

        <div class="flex gap-2 mt-4 pt-4 border-t border-base-200 flex-shrink-0">
          <Button 
            variant="ghost" 
            size="sm" 
            class="flex-1"
            @click="handleClose"
          >
            取消
          </Button>
          <Button 
            variant="ghost" 
            size="sm" 
            class="flex-1"
            :disabled="!tempSelectedModel && !customModel?.trim()"
            @click="handleTestModel"
          >
            <Loader2 v-if="isTesting" :size="14" class="animate-spin mr-1" />
            <CheckCircle v-else :size="14" class="mr-1" />
            {{ isTesting ? '检测中...' : '检测' }}
          </Button>
          <Button 
            variant="primary" 
            size="sm" 
            class="flex-1"
            :disabled="!tempSelectedModel && !customModel?.trim()"
            @click="handleConfirm"
          >
            <Check :size="14" class="mr-1" />
            确认
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(156, 163, 175, 0.3);
  border-radius: 3px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(156, 163, 175, 0.5);
}
</style>
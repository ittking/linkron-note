<script setup>
import { ref, computed, watch } from 'vue'
import { Search, X, RefreshCw, Plus, Check, CheckCircle, XCircle, Loader2 } from 'lucide-vue-next'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'
import { useToast } from '../composables/useToast'

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
const searchQuery = ref('')
const customModel = ref('')
const tempSelectedModel = ref('')
const isTesting = ref(false)

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

function handleLoadModels() {
  emit('load-models', props.provider)
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

  if (!props.provider?.apiKey) {
    showToast('请先配置 API Key', 'error')
    return
  }

  isTesting.value = true
  
  try {
    const startTime = Date.now()
    
    // 构造测试请求
    const apiUrl = props.provider.apiUrl || getDefaultApiUrl(props.provider.provider)
    const response = await fetch(apiUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${props.provider.apiKey}`
      },
      body: JSON.stringify({
        model: modelToTest,
        messages: [{ role: 'user', content: 'Hi' }],
        max_tokens: 1
      })
    })

    const endTime = Date.now()
    const latency = endTime - startTime

    if (response.ok) {
      showToast(`模型连接成功，延时: ${latency}ms`, 'success')
    } else {
      const errorText = await response.text()
      showToast(`模型连接失败: ${response.status} ${errorText.substring(0, 50)}`, 'error')
    }
  } catch (error) {
    showToast(`模型连接失败: ${error.message}`, 'error')
  } finally {
    isTesting.value = false
  }
}

function getDefaultApiUrl(providerName) {
  const urlMap = {
    'openai': 'https://api.openai.com/v1/chat/completions',
    'anthropic': 'https://api.anthropic.com/v1/messages',
    'deepseek': 'https://api.deepseek.com/v1/chat/completions',
    'moonshot': 'https://api.moonshot.cn/v1/chat/completions',
    'zhipu': 'https://open.bigmodel.cn/api/paas/v4/chat/completions',
    'ollama': 'http://localhost:11434/v1/chat/completions'
  }
  return urlMap[providerName] || 'https://api.openai.com/v1/chat/completions'
}

watch(() => props.show, (newVal) => {
  if (newVal) {
    searchQuery.value = ''
    customModel.value = ''
    tempSelectedModel.value = ''
    if (props.provider?.currentModel) {
      tempSelectedModel.value = props.provider.currentModel
      if (!props.models.includes(props.provider.currentModel)) {
        customModel.value = props.provider.currentModel
      }
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
            >
              <RefreshCw :size="14" />
            </Button>
          </div>

          <div class="flex-1 overflow-y-auto min-h-0 max-h-[300px]">
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
                  tempSelectedModel === model || provider?.currentModel === model
                    ? 'bg-primary/10 border-primary text-primary'
                    : 'bg-base-100 border-transparent hover:bg-base-200 hover:border-base-300'
                ]"
              >
                <div class="flex items-center justify-between">
                  <span class="text-sm">{{ model }}</span>
                  <Check 
                    v-if="tempSelectedModel === model || provider?.currentModel === model" 
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
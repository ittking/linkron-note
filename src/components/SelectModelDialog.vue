<script setup>
import { ref, computed, watch } from 'vue'
import { Search, X, RefreshCw, Plus } from 'lucide-vue-next'
import Input from './ui/Input.vue'
import Button from './ui/Button.vue'

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

const searchQuery = ref('')
const customModel = ref('')

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
  emit('close')
}

function handleSelect(model) {
  emit('select', model)
  handleClose()
}

function handleLoadModels() {
  emit('load-models', props.provider)
}

function handleAddCustom() {
  if (!customModel.value.trim()) {
    alert('请输入模型名称')
    return
  }
  emit('add-custom', customModel.value.trim())
  customModel.value = ''
  handleClose()
}

watch(() => props.show, (newVal) => {
  if (newVal) {
    searchQuery.value = ''
    customModel.value = ''
    if (props.provider?.currentModel && !props.models.includes(props.provider.currentModel)) {
      customModel.value = props.provider.currentModel
    }
  } else {
    searchQuery.value = ''
    customModel.value = ''
  }
})
</script>

<template>
  <div v-if="show" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
    <div class="card bg-base-100 shadow-xl w-full max-w-md mx-4 max-h-[80vh] flex flex-col">
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

          <div class="flex-1 overflow-y-auto min-h-0">
            <div v-if="filteredModels.length === 0" class="text-center py-6 text-base-content/40 text-sm">
              暂无模型，请先加载或添加自定义模型
            </div>
            <div v-else class="space-y-1">
              <div 
                v-for="model in filteredModels" 
                :key="model"
                @click="handleSelect(model)"
                class="p-2.5 rounded hover:bg-base-200 cursor-pointer transition-colors border border-transparent hover:border-base-300"
              >
                <div class="flex items-center justify-between">
                  <span class="text-sm">{{ model }}</span>
                  <Plus :size="12" class="text-base-content/40" />
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
      </div>
    </div>
  </div>
</template>
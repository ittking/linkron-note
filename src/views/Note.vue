<script setup>
import { ref, onMounted, onActivated, nextTick } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { ExternalLink, Edit, Trash2, Download } from 'lucide-vue-next'
import NoteCard from '@/components/NoteCard.vue'
import NoteEditor from '@/components/NoteEditor.vue'
import { useNoteStore } from '@/store/noteStore'

const noteStore = useNoteStore()

// 滚动位置保存
const noteListRef = ref(null)
let savedScrollTop = 0
const isNoteListScrolledToTop = ref(true)

const notes = ref([])
const editorContent = ref('')
const isDragging = ref(false)
const dragCounter = ref(0) // 拖拽计数器
const toastMessage = ref('')
const toastVisible = ref(false)
const toastType = ref('info')

// 编辑相关状态
const editingNote = ref(null)
const isEditing = ref(false)

// 笔记详情抽屉
const drawerVisible = ref(false)

// 确认对话框
const confirmVisible = ref(false)
const confirmTitle = ref('')
const confirmContent = ref('')
const confirmOnOk = ref(null)

// 路由离开前保存滚动位置
onBeforeRouteLeave((to, from, next) => {
  if (noteListRef.value) {
    savedScrollTop = noteListRef.value.scrollTop
  }
  next()
})

// 组件激活时恢复滚动位置
onActivated(async () => {
  await nextTick()
  
  // 使用 setTimeout 确保 DOM 完全更新
  setTimeout(() => {
    if (noteListRef.value && savedScrollTop > 0) {
      noteListRef.value.scrollTop = savedScrollTop
    }
  }, 50)
})

// 监听笔记列表滚动
function handleNoteListScroll() {
    if (noteListRef.value) {
        isNoteListScrolledToTop.value = noteListRef.value.scrollTop === 0
    }
}

// 初始化
onMounted(async () => {
    await loadNotes()
})

// 显示提示
function showToast(message, type = 'info') {
    toastMessage.value = message
    toastType.value = type
    toastVisible.value = true
    setTimeout(() => {
        toastVisible.value = false
    }, 3000)
}

// 加载笔记
async function loadNotes() {
    notes.value = await noteStore.getNotes()
}

// 编辑器提交
async function handleEditorSubmit() {
    if (editorContent.value.trim()) {
        if (isEditing.value && editingNote.value) {
            // 编辑模式：更新笔记
            await noteStore.updateNote(editingNote.value.id, {
                content: editorContent.value
            })
            editingNote.value = null
            isEditing.value = false
            editorContent.value = ''
            await loadNotes()
            showToast('笔记更新成功', 'success')
        } else {
            // 创建模式：创建新笔记
            await noteStore.addNote({
                type: 'text',
                content: editorContent.value,
                images: []
            })
            editorContent.value = ''
            await loadNotes()
            showToast('笔记创建成功', 'success')
        }
    }
}

// 图片上传处理
async function handleImageUpload(file) {
    const reader = new FileReader()
    reader.onload = async (e) => {
        await noteStore.addNote({
            type: 'image',
            content: file.name || '图片笔记',
            images: [e.target.result]
        })
        await loadNotes()
        showToast('图片笔记创建成功', 'success')
    }
    reader.readAsDataURL(file)
}

// 拖拽事件处理
function handleDragEnter(e) {
    e.preventDefault()
    dragCounter.value++
    isDragging.value = true
}

function handleDragLeave(e) {
    e.preventDefault()
    dragCounter.value--
    if (dragCounter.value <= 0) {
        isDragging.value = false
        dragCounter.value = 0
    }
}

function handleDragOver(e) {
    e.preventDefault()
}

function handleDrop(e) {
    e.preventDefault()
    isDragging.value = false
    dragCounter.value = 0

    // 优先处理 URL/文本数据
    const textData = e.dataTransfer.getData('text/uri-list') || e.dataTransfer.getData('text/plain')

    if (textData) {
        handleDroppedData(textData)
        return
    }

    // 处理文件
    const files = e.dataTransfer.files
    if (files && files.length > 0) {
        for (let i = 0; i < files.length; i++) {
            handleDroppedFile(files[i])
        }
    }
}

// 处理拖拽数据
function handleDroppedData(data) {
    // 更宽松的 URL 正则表达式，支持查询参数
    const urlRegex = /^(https?:\/\/)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.\-?=&%]*)*\/?$/
    const isUrl = urlRegex.test(data)

    if (isUrl) {
        createLinkNote(data)
    } else {
        createTextNote(data)
    }
}

// 处理拖拽文件
function handleDroppedFile(file) {
    if (file.type.startsWith('image/')) {
        const reader = new FileReader()
        reader.onload = (e) => {
            createImageNote(e.target.result, file.name)
        }
        reader.readAsDataURL(file)
    } else {
        showToast('不支持的文件类型', 'error')
    }
}

// 创建链接笔记
async function createLinkNote(url) {
    try {
        const note = await noteStore.addNote({
            type: 'link',
            content: url,
            sourceUrl: url,
            images: []
        })

        showToast('正在解析链接...', 'info')

        // 模拟链接解析
        setTimeout(async () => {
            await noteStore.updateNote(note.id, {
                content: '这是从链接抓取的内容摘要。在实际应用中，这里会显示网页的正文内容...',
                images: [
                    'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAwIiBoZWlnaHQ9IjMwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjMwMCIgZmlsbD0iIzJhMmEzMiIvPjx0ZXh0IHg9IjUwJSIgeT0iNTAlIiBkb21pbmFudC1iYXNlbGluZT0ibWlkZGxlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjI0IiBmaWxsPSIjNmI2Yjc2Ij5JbWFnZSAxPC90ZXh0Pjwvc3ZnPg==',
                    'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAwIiBoZWlnaHQ9IjMwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjMwMCIgZmlsbD0iIzJhMmEzMiIvPjx0ZXh0IHg9IjUwJSIgeT0iNTAlIiBkb21pbmFudC1iYXNlbGluZT0ibWlkZGxlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjI0IiBmaWxsPSIjNmI2Yjc2Ij5JbWFnZSAyPC90ZXh0Pjwvc3ZnPg=='
                ]
            })
            await loadNotes()
            showToast('笔记创建成功', 'success')
        }, 1000)
    } catch (error) {
        showToast('创建笔记失败', 'error')
    }
}

// 创建图片笔记
async function createImageNote(imageData, fileName) {
    await noteStore.addNote({
        type: 'image',
        content: fileName || '图片笔记',
        images: [imageData]
    })
    await loadNotes()
    showToast('图片笔记创建成功', 'success')
}

// 创建文字笔记
async function createTextNote(text) {
    try {
        await noteStore.addNote({
            type: 'text',
            content: text,
            images: []
        })
        await loadNotes()
        showToast('文字笔记创建成功', 'success')
    } catch (error) {
        showToast('创建笔记失败', 'error')
    }
}

// 卡片点击事件
function handleCardClick(note) {
    // 暂时不做任何操作
}

// 菜单事件
function handleMenuOpen(note) {
    if (note.sourceUrl) {
        window.open(note.sourceUrl, '_blank')
    } else {
        showToast('此笔记没有链接', 'info')
    }
}

function handleMenuEdit(note) {
    editingNote.value = note
    editorContent.value = note.content
    isEditing.value = true
    showToast('进入编辑模式', 'info')
}

function handleMenuDelete(note) {
    confirmTitle.value = '确认删除'
    confirmContent.value = '确定要删除这条笔记吗？'
    confirmOnOk.value = async () => {
        await noteStore.deleteNote(note.id)
        await loadNotes()
        showToast('笔记已删除', 'success')
    }
    confirmVisible.value = true
}

// 确认对话框回调
function handleConfirmOk() {
    if (confirmOnOk.value) {
        confirmOnOk.value()
    }
    confirmVisible.value = false
}

// 取消编辑
function handleCancelEdit() {
    editingNote.value = null
    isEditing.value = false
    editorContent.value = ''
    showToast('已取消编辑', 'info')
}
</script>

<template>
    <div class="h-full flex flex-col max-w-200 mx-auto" @dragenter="handleDragEnter" @dragleave="handleDragLeave"
        @dragover="handleDragOver" @drop="handleDrop">
        <!-- 编辑器区域 -->
        <div class="px-4 py-3">
            <NoteEditor
                v-model="editorContent"
                :placeholder="isEditing ? '编辑笔记...' : '现在的想法是...'"
                :is-scrolled-to-top="isNoteListScrolledToTop"
                :is-editing="isEditing"
                @submit="handleEditorSubmit"
                @image-upload="handleImageUpload"
            >
                <template #actions>
                    <button
                        v-if="isEditing"
                        @click="handleCancelEdit"
                        class="px-3 h-7 rounded-md flex items-center justify-center transition-all duration-200 bg-base-300 text-base-content/60 hover:bg-base-200 hover:text-base-content text-xs"
                        title="取消编辑"
                    >
                        取消
                    </button>
                </template>
            </NoteEditor>
        </div>

        <!-- 笔记列表 -->
        <div class="flex-1 overflow-hidden">
            <div 
                ref="noteListRef"
                class="p-3 h-full overflow-y-auto no-scrollbar"
                @scroll="handleNoteListScroll"
            >
                <div v-if="notes.length === 0" class="flex flex-col items-center justify-center h-full text-base-content/40 text-center p-5">
                    <div class="text-5xl mb-4 opacity-50">📝</div>
                    <div class="text-base font-medium mb-2 text-base-content/60">暂无笔记</div>
                    <div class="text-sm leading-relaxed max-w-[240px]">拖拽链接、文字或图片到这里创建笔记</div>
                </div>

                <NoteCard v-for="note in notes" :key="note.id" :note="note" @click="handleCardClick"
                    @open="handleMenuOpen" @edit="handleMenuEdit" @delete="handleMenuDelete" />
            </div>
        </div>

        <!-- 拖拽遮罩 -->
        <div
            v-if="isDragging"
            class="fixed inset-0 bg-primary/5 border-2 border-dashed border-primary flex flex-col items-center justify-center z-[9999] transition-opacity duration-200"
            @dragenter="handleDragEnter"
            @dragleave="handleDragLeave"
            @dragover="handleDragOver"
            @drop="handleDrop"
        >
            <Download :size="48" class="text-primary mb-4 animate-bounce" />
            <div class="text-base font-medium text-primary mb-2">释放以创建笔记</div>
            <div class="text-sm text-base-content/60">支持链接、文字、图片</div>
        </div>

        <!-- Toast 提示 -->
        <div :class="['fixed top-4 right-4 z-[200] px-4 py-3 rounded-lg shadow-lg transition-all duration-300', toastVisible ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0', toastType === 'success' ? 'bg-success text-success-content' : toastType === 'error' ? 'bg-error text-error-content' : 'bg-info text-info-content']">
            {{ toastMessage }}
        </div>

        <!-- 确认对话框 -->
        <dialog :open="confirmVisible" class="modal">
            <div class="modal-box bg-base-200 border border-base-300">
                <h3 class="font-bold text-lg text-base-content">{{ confirmTitle }}</h3>
                <p class="py-4 text-base-content/60">{{ confirmContent }}</p>
                <div class="modal-action">
                    <button @click="confirmVisible = false" class="btn btn-ghost text-base-content/60 hover:text-base-content">取消</button>
                    <button @click="handleConfirmOk" class="btn btn-error text-error-content">删除</button>
                </div>
            </div>
            <form method="dialog" class="modal-backdrop bg-black/50">
                <button @click="confirmVisible = false"></button>
            </form>
        </dialog>
    </div>
</template>
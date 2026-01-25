<script setup>
import { ref, onMounted } from 'vue'
import { Search, ExternalLink, Edit, Trash2, X } from 'lucide-vue-next'
import NoteCard from '@/components/NoteCard.vue'
import { useNoteStore } from '@/store/noteStore'

const noteStore = useNoteStore()

const notes = ref([])
const searchKeyword = ref('')
const isDragging = ref(false)
const toastMessage = ref('')
const toastVisible = ref(false)
const toastType = ref('info')

// 笔记详情抽屉
const drawerVisible = ref(false)
const drawerTitle = ref('')
const drawerContent = ref('')
const drawerImages = ref([])
const drawerSourceUrl = ref('')
const editingNoteId = ref(null)
const isEditMode = ref(false)

// 确认对话框
const confirmVisible = ref(false)
const confirmTitle = ref('')
const confirmContent = ref('')
const confirmOnOk = ref(null)

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

// 搜索笔记
async function handleSearch() {
    if (searchKeyword.value.trim()) {
        notes.value = await noteStore.searchNotes(searchKeyword.value)
    } else {
        await loadNotes()
    }
}

// 拖拽事件处理
function handleDragEnter(e) {
    e.preventDefault()
    isDragging.value = true
}

function handleDragLeave(e) {
    e.preventDefault()
    isDragging.value = false
}

function handleDragOver(e) {
    e.preventDefault()
}

function handleDrop(e) {
    e.preventDefault()
    isDragging.value = false

    const items = e.dataTransfer.items
    let hasProcessedUrl = false

    if (items) {
        for (let i = 0; i < items.length; i++) {
            const item = items[i]

            // 优先处理 string 类型的数据（URL、文本）
            if (item.kind === 'string' && !hasProcessedUrl) {
                item.getAsString((data) => {
                    const urlRegex = /^(https?:\/\/)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.-]*)*\/?$/
                    if (urlRegex.test(data)) {
                        hasProcessedUrl = true
                    }
                    handleDroppedData(data)
                })
            }
            // 如果已经处理了 URL，就跳过文件类型的处理
            else if (item.kind === 'file' && !hasProcessedUrl) {
                const file = item.getAsFile()
                handleDroppedFile(file)
            }
        }
    }
}

// 处理拖拽数据
function handleDroppedData(data) {
    const urlRegex = /^(https?:\/\/)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.-]*)*\/?$/
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
    const note = await noteStore.addNote({
        type: 'link',
        title: '加载中...',
        content: '',
        sourceUrl: url,
        images: []
    })

    showToast('正在解析链接...', 'info')

    // 模拟链接解析
    setTimeout(async () => {
        await noteStore.updateNote(note.id, {
            title: '网页标题 - ' + new Date().toLocaleDateString(),
            content: '这是从链接抓取的内容摘要。在实际应用中，这里会显示网页的正文内容...',
            images: [
                'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAwIiBoZWlnaHQ9IjMwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjMwMCIgZmlsbD0iIzJhMmEzMiIvPjx0ZXh0IHg9IjUwJSIgeT0iNTAlIiBkb21pbmFudC1iYXNlbGluZT0ibWlkZGxlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjI0IiBmaWxsPSIjNmI2Yjc2Ij5JbWFnZSAxPC90ZXh0Pjwvc3ZnPg==',
                'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAwIiBoZWlnaHQ9IjMwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjMwMCIgZmlsbD0iIzJhMmEzMiIvPjx0ZXh0IHg9IjUwJSIgeT0iNTAlIiBkb21pbmFudC1iYXNlbGluZT0ibWlkZGxlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBmb250LXNpemU9IjI0IiBmaWxsPSIjNmI2Yjc2Ij5JbWFnZSAyPC90ZXh0Pjwvc3ZnPg=='
            ]
        })
        await loadNotes()
        showToast('笔记创建成功', 'success')
    }, 1000)
}

// 创建图片笔记
async function createImageNote(imageData, fileName) {
    await noteStore.addNote({
        type: 'image',
        title: fileName || '图片笔记',
        content: '',
        images: [imageData]
    })
    await loadNotes()
    showToast('图片笔记创建成功', 'success')
}

// 创建文字笔记
async function createTextNote(text) {
    await noteStore.addNote({
        type: 'text',
        title: text.substring(0, 30) + (text.length > 30 ? '...' : ''),
        content: text,
        images: []
    })
    await loadNotes()
    showToast('文字笔记创建成功', 'success')
}

// 卡片点击事件
function handleCardClick(note) {
    openDrawer(note)
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
    openDrawer(note, true)
}

function handleMenuDelete(note) {
    confirmTitle.value = '确认删除'
    confirmContent.value = `确定要删除笔记"${note.title}"吗？`
    confirmOnOk.value = async () => {
        await noteStore.deleteNote(note.id)
        await loadNotes()
        showToast('笔记已删除', 'success')
    }
    confirmVisible.value = true
}

// 打开抽屉
async function openDrawer(note, edit = false) {
    editingNoteId.value = note.id
    isEditMode.value = edit
    drawerTitle.value = edit ? `编辑: ${note.title}` : note.title
    drawerContent.value = note.content || ''
    drawerImages.value = note.images || []
    drawerSourceUrl.value = note.sourceUrl || ''
    drawerVisible.value = true
}

// 保存编辑
async function saveDrawer() {
    if (editingNoteId.value) {
        await noteStore.updateNote(editingNoteId.value, {
            title: drawerTitle.value,
            content: drawerContent.value,
            images: drawerImages.value
        })
        await loadNotes()
        showToast('笔记已更新', 'success')
    }
    drawerVisible.value = false
}

// 关闭抽屉
function closeDrawer() {
    drawerVisible.value = false
    editingNoteId.value = null
    isEditMode.value = false
}

// 确认对话框回调
function handleConfirmOk() {
    if (confirmOnOk.value) {
        confirmOnOk.value()
    }
    confirmVisible.value = false
}
</script>

<template>
    <div class="h-full flex flex-col" @dragenter="handleDragEnter" @dragleave="handleDragLeave"
        @dragover="handleDragOver" @drop="handleDrop">
        <!-- 搜索栏 -->
        <div class="px-4 py-3 border-b border-base-300">
            <div class="relative">
                <Search :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40" />
                <input
                    v-model="searchKeyword"
                    @input="handleSearch"
                    type="text"
                    placeholder="搜索笔记..."
                    class="w-full pl-9 pr-3 py-2 bg-base-200 border border-base-300 rounded-lg text-base-content placeholder-base-content/40 focus:outline-none focus:border-primary transition-colors"
                />
            </div>
        </div>

        <!-- 笔记列表 -->
        <div class="flex-1 overflow-hidden">
            <div class="p-3 h-full overflow-y-auto no-scrollbar">
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
            :class="['fixed inset-0 bg-primary/5 border-2 border-dashed border-primary flex flex-col items-center justify-center z-[100] opacity-0 pointer-events-none transition-opacity duration-200', { 'opacity-100 pointer-events-auto': isDragging }]">
            <div class="text-5xl text-primary mb-4 animate-bounce">📥</div>
            <div class="text-base font-medium text-primary mb-2">释放以创建笔记</div>
            <div class="text-sm text-base-content/60">支持链接、文字、图片</div>
        </div>

        <!-- 笔记详情抽屉 -->
        <div :class="['fixed inset-x-0 bottom-0 z-[50] transition-transform duration-300', drawerVisible ? 'translate-y-0' : 'translate-y-full']">
            <div class="bg-base-200 border-t border-base-300 rounded-t-2xl max-h-[90vh] overflow-y-auto">
                <!-- 抽屉头部 -->
                <div class="sticky top-0 bg-base-200 border-b border-base-300 px-4 py-3 flex items-center justify-between">
                    <h3 class="text-lg font-medium text-base-content">{{ drawerTitle }}</h3>
                    <div class="flex items-center gap-2">
                        <button v-if="isEditMode" @click="saveDrawer" class="btn btn-sm btn-success text-success-content">
                            保存
                        </button>
                        <button @click="closeDrawer" class="btn btn-sm btn-ghost text-base-content/60 hover:text-base-content">
                            <X :size="18" />
                        </button>
                    </div>
                </div>

                <!-- 抽屉内容 -->
                <div class="p-4">
                    <!-- 图片展示 -->
                    <div v-if="drawerImages.length > 0" class="flex gap-3 mb-4 flex-wrap">
                        <img v-for="(img, index) in drawerImages" :key="index" :src="img"
                            class="w-full max-w-[180px] rounded-lg object-cover border border-base-300" alt="Note image" />
                    </div>

                    <!-- 内容 -->
                    <div v-if="drawerContent" class="text-sm leading-[1.7] text-base-content">
                        <div v-if="!isEditMode">{{ drawerContent }}</div>
                        <textarea
                            v-else
                            v-model="drawerContent"
                            placeholder="输入笔记内容..."
                            rows="8"
                            class="w-full p-3 bg-base-300 border border-base-300 rounded-lg text-base-content placeholder-base-content/40 focus:outline-none focus:border-primary transition-colors resize-none"
                        ></textarea>
                    </div>

                    <!-- 来源链接 -->
                    <div v-if="drawerSourceUrl" class="mt-4 pt-4 border-t border-base-300 text-xs text-base-content/40">
                        来源: <a :href="drawerSourceUrl" target="_blank" class="text-primary hover:underline flex items-center gap-1">
                            {{ drawerSourceUrl }}
                            <ExternalLink :size="12" />
                        </a>
                    </div>
                </div>
            </div>
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
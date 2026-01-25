<script setup>
import { ref, onMounted } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { SearchOutlined } from '@ant-design/icons-vue'
import NoteCard from '@/components/NoteCard.vue'
import { useNoteStore } from '@/store/noteStore'

const noteStore = useNoteStore()

const notes = ref([])
const searchKeyword = ref('')
const isDragging = ref(false)
let draggedData = null

// 笔记详情抽屉
const drawerVisible = ref(false)
const drawerTitle = ref('')
const drawerContent = ref('')
const drawerImages = ref([])
const drawerSourceUrl = ref('')
const editingNoteId = ref(null)
const isEditMode = ref(false)

// 初始化
onMounted(async () => {
    await loadNotes()
})

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
        message.error('不支持的文件类型')
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

    message.loading('正在解析链接...', 1)

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
        message.success('笔记创建成功')
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
    message.success('图片笔记创建成功')
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
    message.success('文字笔记创建成功')
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
        message.info('此笔记没有链接')
    }
}

function handleMenuEdit(note) {
    openDrawer(note, true)
}

function handleMenuDelete(note) {
    Modal.confirm({
        title: '确认删除',
        content: `确定要删除笔记"${note.title}"吗？`,
        okText: '删除',
        okType: 'danger',
        cancelText: '取消',
        onOk: async () => {
            await noteStore.deleteNote(note.id)
            await loadNotes()
            message.success('笔记已删除')
        }
    })
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
        message.success('笔记已更新')
    }
    drawerVisible.value = false
}

// 关闭抽屉
function closeDrawer() {
    drawerVisible.value = false
    editingNoteId.value = null
    isEditMode.value = false
}
</script>

<template>
    <div class="h-full flex flex-col" @dragenter="handleDragEnter" @dragleave="handleDragLeave"
        @dragover="handleDragOver" @drop="handleDrop">
        <!-- 搜索栏 -->
        <div class="px-4 py-3 border-b border-[#2a2a32]">
            <a-input v-model:value="searchKeyword" placeholder="搜索笔记..." @input="handleSearch" allow-clear>
                <template #prefix>
                    <SearchOutlined class="text-[#4a4a55]" />
                </template>
            </a-input>
        </div>

        <!-- 笔记列表 -->
        <div class="flex-1 overflow-hidden">
            <div class="p-3 h-full overflow-y-auto">
                <a-empty v-if="notes.length === 0" :image="false" description="">
                    <template #description>
                        <div class="flex flex-col items-center justify-center h-full text-[#4a4a55] text-center p-5">
                            <div class="text-5xl mb-4 opacity-50">📝</div>
                            <div class="text-base font-medium mb-2 text-[#6b6b76]">暂无笔记</div>
                            <div class="text-sm leading-relaxed max-w-[240px]">拖拽链接、文字或图片到这里创建笔记</div>
                        </div>
                    </template>
                </a-empty>

                <NoteCard v-for="note in notes" :key="note.id" :note="note" @click="handleCardClick"
                    @open="handleMenuOpen" @edit="handleMenuEdit" @delete="handleMenuDelete" />
            </div>
        </div>

        <!-- 拖拽遮罩 -->
        <div
            :class="['fixed inset-0 bg-[rgba(0,255,136,0.05)] border-2 border-dashed border-[#00ff88] flex flex-col items-center justify-center z-[100] opacity-0 pointer-events-none transition-opacity duration-200', { 'opacity-100 pointer-events-auto': isDragging }]">
            <div class="text-5xl text-[#00ff88] mb-4 animate-bounce">📥</div>
            <div class="text-base font-medium text-[#00ff88] mb-2">释放以创建笔记</div>
            <div class="text-sm text-[#6b6b76]">支持链接、文字、图片</div>
        </div>

        <!-- 笔记详情抽屉 -->
        <a-drawer v-model:open="drawerVisible" :title="drawerTitle" placement="bottom" height="90%"
            @close="closeDrawer">
            <template #extra>
                <a-button v-if="isEditMode" type="primary" @click="saveDrawer">
                    保存
                </a-button>
            </template>

            <!-- 图片展示 -->
            <div v-if="drawerImages.length > 0" class="flex gap-3 mb-4 flex-wrap">
                <img v-for="(img, index) in drawerImages" :key="index" :src="img"
                    class="w-full max-w-[180px] rounded-lg object-cover border border-[#2a2a32]" alt="Note image" />
            </div>

            <!-- 内容 -->
            <div v-if="drawerContent" class="text-sm leading-[1.7] text-[#e8e8ed]">
                <div v-if="!isEditMode">{{ drawerContent }}</div>
                <a-textarea v-else v-model:value="drawerContent" placeholder="输入笔记内容..."
                    :auto-size="{ minRows: 8, maxRows: 20 }" />
            </div>

            <!-- 来源链接 -->
            <div v-if="drawerSourceUrl" class="mt-4 pt-4 border-t border-[#2a2a32] text-xs text-[#4a4a55]">
                来源: <a :href="drawerSourceUrl" target="_blank" class="text-[#60a5fa] hover:underline">{{
                    drawerSourceUrl }}</a>
            </div>
        </a-drawer>
    </div>
</template>
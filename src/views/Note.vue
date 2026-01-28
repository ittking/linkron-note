<script setup>
import { ref, onMounted, onActivated, nextTick } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { Download, FileText } from 'lucide-vue-next'
import NoteCard from '@/components/NoteCard.vue'
import NoteEditor from '@/components/NoteEditor.vue'
import { useNoteStore } from '@/store/noteStore'
import { saveFile } from '@/utils/fileUpload'
import { extractTextFromFile, isSupportedFileType, getFileTypeDescription } from '@/utils/textExtraction'
import { scrapeWebPage, isValidUrl, formatWebPageToNote } from '@/utils/webScraper'
import { extractUrlFromUrlFile } from '@/utils/urlFileParser'

const noteStore = useNoteStore()

// 编辑器引用
const noteEditorRef = ref(null)

// 滚动位置保存
const noteListRef = ref(null)
let savedScrollTop = 0
const isNoteListScrolledToTop = ref(true)

// 编辑器高度控制状态
const EDITOR_THRESHOLD = 150 // 切换编辑器高度的滚动阈值
let lastScrollTop = 0
let isScrollingDown = false // 滚动方向标志

// 优化的编辑器高度更新逻辑
function updateEditorHeight(scrollTop) {
    // 判断滚动方向
    isScrollingDown = scrollTop > lastScrollTop
    lastScrollTop = scrollTop

    // 使用方向性阈值，避免在临界点附近反复切换
    if (isScrollingDown) {
        // 向下滚动：超过阈值才缩小编辑器
        if (scrollTop > EDITOR_THRESHOLD && isNoteListScrolledToTop.value) {
            isNoteListScrolledToTop.value = false
        }
    } else {
        // 向上滚动：回到阈值以下才恢复编辑器
        if (scrollTop < EDITOR_THRESHOLD / 2 && !isNoteListScrolledToTop.value) {
            isNoteListScrolledToTop.value = true
        }
    }
}

const notes = ref([])
const editorContent = ref('')
const dragCounter = ref(0) // 拖拽计数器
const isDragging = ref(false)
const toastMessage = ref('')
const toastVisible = ref(false)
const toastType = ref('info')

// 分页相关状态
const currentPage = ref(1)
const pageSize = ref(20)
const hasMore = ref(true)
const isLoading = ref(false)

// 编辑相关状态
const editingNote = ref(null)
const isEditing = ref(false)
const shouldClearEditor = ref(false) // 添加标志位用于强制清空编辑器

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
        const scrollTop = noteListRef.value.scrollTop

        // 滚动到底部时加载更多（实时检测，不防抖）
        const { scrollHeight, clientHeight } = noteListRef.value
        const distanceToBottom = scrollHeight - scrollTop - clientHeight

        // 距离底部小于 100px 时加载更多
        if (distanceToBottom < 100 && hasMore.value && !isLoading.value) {
            loadNotes()
        }

        // 使用优化的编辑器高度更新逻辑
        updateEditorHeight(scrollTop)
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
async function loadNotes(reset = false) {
    if (isLoading.value) return

    if (reset) {
        currentPage.value = 1
        notes.value = []
        hasMore.value = true
    }

    if (!hasMore.value) return

    isLoading.value = true

    try {
        const newNotes = await noteStore.getNotes(currentPage.value, pageSize.value)

        if (newNotes.length < pageSize.value) {
            hasMore.value = false
        }

        if (reset) {
            notes.value = newNotes
        } else {
            notes.value = [...notes.value, ...newNotes]
        }

        currentPage.value++
    } catch (error) {
        console.error('Failed to load notes:', error)
        showToast('加载笔记失败', 'error')
    } finally {
        isLoading.value = false
    }
}

// 编辑器提交
async function handleEditorSubmit(noteData) {
    // noteData 包含 { content, images }
    const content = noteData?.content || editorContent.value
    const images = noteData?.images || []

    if (content.trim() || images.length > 0) {
        if (isEditing.value && editingNote.value) {
            // 编辑模式：更新笔记
            try {
                await noteStore.updateNote(editingNote.value.id, {
                    content: content,
                    images: images
                })
                // 直接在数组中更新笔记
                const index = notes.value.findIndex(n => n.id === editingNote.value.id)
                if (index !== -1) {
                    notes.value[index] = {
                        ...notes.value[index],
                        content: content,
                        images: images
                    }
                }
                editingNote.value = null
                isEditing.value = false
                editorContent.value = ''
                showToast('笔记更新成功', 'success')
            } catch (error) {
                console.error('Failed to update note:', error)
                showToast('笔记更新失败：' + error.message, 'error')
            }
        } else {
            // 创建模式：创建新笔记
            const newNote = await noteStore.addNote({
                type: 'text',
                content: content,
                images: images
            })
            // 直接在数组开头添加新笔记
            notes.value.unshift(newNote)
            editorContent.value = ''
            showToast('笔记创建成功', 'success')
        }
    }
}

// 拖拽事件处理
function handleDragEnter(e) {
    e.preventDefault()
    dragCounter.value++
    isDragging.value = true
}

function handleDragLeave(e) {
    // 简化：如果鼠标离开容器且不是进入子元素，则隐藏
    if (e.target === e.currentTarget) {
        isDragging.value = false
    }
}

function handleDragOver(e) {
    e.preventDefault()
}

function handleDrop(e) {
    e.preventDefault()
    e.stopPropagation()

    // 简化：直接处理，不需要 dragCounter
    isDragging.value = false

    // 优先处理文件
    const files = e.dataTransfer.files
    if (files && files.length > 0) {
        for (let i = 0; i < files.length; i++) {
            handleDroppedFile(files[i])
        }
        return
    }

    // 处理 URL/文本数据
    const textData = e.dataTransfer.getData('text/uri-list') || e.dataTransfer.getData('text/plain')

    if (textData) {
        handleDroppedData(textData)
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
async function handleDroppedFile(file) {
    const workDirectory = await noteStore.getWorkDirectory()

    // .url 文件 - 解析文件内容获取 URL
    if (file.name.endsWith('.url')) {
        try {
            const url = await extractUrlFromUrlFile(file)
            if (url && isValidUrl(url)) {
                createLinkNote(url)
            } else {
                showToast('无法从 .url 文件中提取有效的 URL', 'error')
            }
        } catch (error) {
            console.error('解析 .url 文件失败:', error)
            showToast('解析 .url 文件失败: ' + error.message, 'error')
        }
    }
    // 文档文件（txt、md）
    else if (isSupportedFileType(file.name)) {
        try {
            showToast(`正在读取${getFileTypeDescription(file.name)}...`, 'info')

            // 保存文件到工作目录
            const savedPath = await saveFile(file, 'file', workDirectory)

            // 提取文本内容
            const content = await extractTextFromFile(file, savedPath, workDirectory)

            // 创建附件笔记，type='file'，extractUrl 存储文件 URL
            const htmlContent = `<p>${content.replace(/\n/g, '<br>')}</p>`

            const newNote = await noteStore.addNote({
                type: 'file',
                content: htmlContent,
                extractUrl: savedPath
            })
            notes.value.unshift(newNote)
            showToast(`${getFileTypeDescription(file.name)}笔记创建成功`, 'success')
        } catch (error) {
            console.error('文档处理失败:', error)
            showToast('文档处理失败: ' + error.message, 'error')
        }
    }
    // 不支持的文件类型
    else {
        showToast(`不支持的文件类型: ${file.name}`, 'error')
    }
}

// 创建链接笔记
async function createLinkNote(url) {
    if (!isValidUrl(url)) {
        showToast('无效的 URL 格式', 'error')
        return
    }

    try {
        showToast('正在抓取网页信息...', 'info')

        // 抓取网页信息
        const pageInfo = await scrapeWebPage(url)

        // 格式化为笔记内容
        const content = formatWebPageToNote(pageInfo)

        // 创建链接笔记，包含爬取的图片
        const newNote = await noteStore.addNote({
            type: 'link',
            content: content,
            sourceUrl: url,
            images: pageInfo.images || [] // 添加图片数组
        })

        notes.value.unshift(newNote)
        showToast('链接笔记创建成功', 'success')
    } catch (error) {
        console.error('链接抓取失败:', error)
        showToast('链接抓取失败: ' + error.message, 'error')
    }
}

// 创建文字笔记
async function createTextNote(text) {
    try {
        const newNote = await noteStore.addNote({
            type: 'text',
            content: text
        })
        // 直接在数组开头添加新笔记
        notes.value.unshift(newNote)
        showToast('文字笔记创建成功', 'success')
    } catch (error) {
        showToast('创建笔记失败', 'error')
    }
}

// 卡片点击
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
    shouldClearEditor.value = false // 重置清空标志
    // 延迟一帧设置 editorContent，确保 NoteEditor 组件已经挂载
    nextTick(() => {
        editorContent.value = note.content
    })
    isEditing.value = true
    // images 会通过 props 传递给 NoteEditor
    showToast('进入编辑模式', 'info')
}

function handleMenuDelete(note) {
    confirmTitle.value = '确认删除'
    confirmContent.value = '确定要删除这条笔记吗？'
    confirmOnOk.value = async () => {
        await noteStore.deleteNote(note.id)
        // 直接从当前数组中移除笔记，避免重新加载导致的空白
        notes.value = notes.value.filter(n => n.id !== note.id)
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
    shouldClearEditor.value = true // 设置清空标志
    // 清理被删除的图片列表（取消编辑时不需要实际删除文件）
    if (noteEditorRef.value?.clearDeletedImages) {
        noteEditorRef.value.clearDeletedImages()
    }
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
            <NoteEditor ref="noteEditorRef" v-model="editorContent" placeholder="现在的想法是..." :is-scrolled-to-top="isNoteListScrolledToTop"
                :is-editing="isEditing" :images="editingNote?.images || []" :should-clear="shouldClearEditor"
                @submit="handleEditorSubmit">
                <template #actions>
                    <button v-if="isEditing" @click="handleCancelEdit"
                        class="px-3 h-7 rounded-md flex items-center justify-center transition-all duration-200 bg-base-300 text-base-content/60 hover:bg-base-200 hover:text-base-content text-xs"
                        title="取消编辑">
                        取消
                    </button>
                </template>
            </NoteEditor>
        </div>

        <!-- 笔记列表 -->
        <div class="flex-1 overflow-hidden">
            <div ref="noteListRef" class="p-3 h-full overflow-y-auto no-scrollbar" @scroll="handleNoteListScroll">
                <div v-if="notes.length === 0"
                    class="flex flex-col items-center justify-center h-full text-base-content/40 text-center p-5">
                    <FileText :size="64" class="mb-4 opacity-50" />
                    <div class="text-base font-medium mb-2 text-base-content/60">暂无笔记</div>
                    <div class="text-sm leading-relaxed max-w-[240px]">拖拽链接或文字到这里创建笔记</div>
                </div>

                <NoteCard v-for="note in notes" :key="note.id" :note="note" @click="handleCardClick"
                    @open="handleMenuOpen" @edit="handleMenuEdit" @delete="handleMenuDelete" />

                <!-- Loading 组件 -->
                <div v-if="isLoading" class="flex justify-center py-4">
                    <span class="loading loading-spinner text-primary"></span>
                </div>
            </div>
        </div>

        <!-- 拖拽遮罩 -->
        <div v-if="isDragging"
            class="fixed inset-0 bg-primary/5 border-2 border-dashed border-primary flex flex-col items-center justify-center z-[9999] transition-opacity duration-200"
            @dragenter="handleDragEnter" @dragleave="handleDragLeave" @dragover="handleDragOver" @drop="handleDrop">
            <Download :size="48" class="text-primary mb-4 animate-bounce" />
            <div class="text-base font-medium text-primary mb-2">释放以创建笔记</div>
            <div class="text-sm text-base-content/60">支持链接、文字</div>
        </div>

        <!-- Toast 提示 -->
        <div
            :class="['toast toast-end z-[200] px-4 py-3 rounded-lg shadow-lg transition-all duration-300', toastVisible ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0', toastType === 'success' ? 'bg-success text-success-content' : toastType === 'error' ? 'bg-error text-error-content' : 'bg-info text-info-content']">
            {{ toastMessage }}
        </div>

        <!-- 确认对话框 -->
        <dialog :open="confirmVisible" class="modal">
            <div class="modal-box bg-base-200 border border-base-300">
                <h3 class="font-bold text-lg text-base-content">{{ confirmTitle }}</h3>
                <p class="py-4 text-base-content/60">{{ confirmContent }}</p>
                <div class="modal-action">
                    <button @click="confirmVisible = false"
                        class="btn btn-ghost text-base-content/60 hover:text-base-content">取消</button>
                    <button @click="handleConfirmOk" class="btn btn-error text-error-content">删除</button>
                </div>
            </div>
            <form method="dialog" class="modal-backdrop bg-black/50">
                <button @click="confirmVisible = false"></button>
            </form>
        </dialog>
    </div>
</template>
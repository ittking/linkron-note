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
import { scrapeWebPage, isValidUrl } from '@/utils/webScraper'
import { extractUrlFromUrlFile } from '@/utils/urlFileParser'

const noteStore = useNoteStore()

// 编辑器引用
const noteEditorRef = ref(null)
const editorContainerRef = ref(null)

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
const isDragging = ref(false) // 拖拽状态
const isProcessing = ref(false) // 处理拖拽数据状态
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

// 判断放置位置是否在编辑器范围内
function isDropInEditor(clientX, clientY) {
    if (!editorContainerRef.value) return false
    
    const rect = editorContainerRef.value.getBoundingClientRect()
    return clientX >= rect.left && clientX <= rect.right &&
           clientY >= rect.top && clientY <= rect.bottom
}

// 拖拽事件处理
let dragCounter = 0

function handleDragEnter(e) {
    e.preventDefault()
    dragCounter++
    isDragging.value = true
}

function handleDragLeave(e) {
    e.preventDefault()
    dragCounter--
    if (dragCounter === 0) {
        isDragging.value = false
    }
}

function handleDragOver(e) {
    e.preventDefault()
}

function handleDrop(e) {
    e.preventDefault()
    e.stopPropagation()

    // 重置计数器和拖拽状态
    dragCounter = 0
    isDragging.value = false

    // 判断放置位置
    const dropInEditor = isDropInEditor(e.clientX, e.clientY)
    console.log('Drop position:', { clientX: e.clientX, clientY: e.clientY, dropInEditor })

    // 优先处理文件
    const files = e.dataTransfer.files
    if (files && files.length > 0) {
        for (let i = 0; i < files.length; i++) {
            if (dropInEditor) {
                console.log('Handle file to editor')
                handleFileToEditor(files[i])
            } else {
                console.log('Handle file to create note')
                handleDroppedFile(files[i])
            }
        }
        return
    }

    // 处理 URL/文本数据
    const textData = e.dataTransfer.getData('text/uri-list') || e.dataTransfer.getData('text/plain')

    if (textData) {
        if (dropInEditor) {
            console.log('Handle data to editor')
            handleDataToEditor(textData)
        } else {
            console.log('Handle data to create note')
            handleDroppedData(textData)
        }
    }
}

// 将文件添加到编辑器
async function handleFileToEditor(file) {
    if (file.name.endsWith('.url')) {
        showToast('请将 .url 文件拖拽到笔记列表创建笔记', 'info')
        return
    }

    const workDirectory = await noteStore.getWorkDirectory()

    // 检查是否是支持的文档文件
    if (isSupportedFileType(file.name)) {
        showToast(`正在读取${getFileTypeDescription(file.name)}...`, 'info')
        isProcessing.value = true
        try {
            // 保存文件到工作目录
            const savedPath = await saveFile(file, 'file', workDirectory)
            
            // 提取文本内容
            const content = await extractTextFromFile(file, savedPath, workDirectory)
            
            // 将内容添加到编辑器
            const htmlContent = content.replace(/\n/g, '<br>')
            editorContent.value += (editorContent.value ? '<br>' : '') + `<p>${htmlContent}</p>`
            showToast(`${getFileTypeDescription(file.name)}内容已添加到编辑器`, 'success')
        } catch (error) {
            showToast(`读取${getFileTypeDescription(file.name)}失败: ${error.message}`, 'error')
        } finally {
            isProcessing.value = false
        }
    } else {
        showToast(`不支持的文件类型，请拖拽到笔记列表创建笔记`, 'info')
    }
}

// 将数据添加到编辑器
async function handleDataToEditor(data) {
    // 更宽松的 URL 正则表达式，支持查询参数
    const urlRegex = /^(https?:\/\/)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.\-?=&%]*)*\/?$/
    const isUrl = urlRegex.test(data)

    if (isUrl) {
        try {
            isProcessing.value = true
            
            // 抓取网页信息和图片
            const { content, images } = await scrapeWebPage(data)
            
            // 将内容添加到编辑器
            editorContent.value += (editorContent.value ? '<br>' : '') + content
            
            // 将图片添加到编辑器
            if (images && images.length > 0 && noteEditorRef.value?.addImages) {
                noteEditorRef.value.addImages(images)
            }
            
            showToast('网页内容已添加到编辑器', 'success')
        } catch (error) {
            showToast('网页抓取失败: ' + error.message, 'error')
        } finally {
            isProcessing.value = false
        }
    } else {
        // 普通文本
        editorContent.value += (editorContent.value ? '<br>' : '') + data
        showToast('文本已添加到编辑器', 'success')
    }
}

// 处理拖拽数据（创建笔记）
async function handleDroppedData(data) {
    // 更宽松的 URL 正则表达式，支持查询参数
    const urlRegex = /^(https?:\/\/)?([\da-z\.-]+)\.([a-z\.]{2,6})([\/\w \.\-?=&%]*)*\/?$/
    const isUrl = urlRegex.test(data)

    if (isUrl) {
        await createLinkNote(data)
    } else {
        await createTextNote(data)
    }
}

// 处理拖拽文件（创建笔记）
async function handleDroppedFile(file) {
    const workDirectory = await noteStore.getWorkDirectory()

    // .url 文件 - 解析文件内容获取 URL
    if (file.name.endsWith('.url')) {
        isProcessing.value = true
        try {
            const url = await extractUrlFromUrlFile(file)
            if (url && isValidUrl(url)) {
                await createLinkNote(url)
            } else {
                showToast('无法从 .url 文件中提取有效的 URL', 'error')
            }
        } catch (error) {
            console.error('解析 .url 文件失败:', error)
            showToast('解析 .url 文件失败: ' + error.message, 'error')
        } finally {
            isProcessing.value = false
        }
    }
    // 文档文件（txt、md）
    else if (isSupportedFileType(file.name)) {
        try {
            isProcessing.value = true

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
        } finally {
            isProcessing.value = false
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
        isProcessing.value = true

        // 抓取网页信息和图片
        const { content, images } = await scrapeWebPage(url)

        // 创建链接笔记，包含爬取的图片
        const newNote = await noteStore.addNote({
            type: 'link',
            content: content,
            sourceUrl: url,
            images: images || [] // 添加图片数组
        })

        notes.value.unshift(newNote)
        showToast('链接笔记创建成功', 'success')
    } catch (error) {
        console.error('链接抓取失败:', error)
        showToast('链接抓取失败: ' + error.message, 'error')
    } finally {
        isProcessing.value = false
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
        <div ref="editorContainerRef" class="px-4 py-3 relative">
            <!-- 编辑器拖拽遮罩 -->
            <div v-if="isDragging"
                class="absolute inset-0 bg-primary/20 border-2 border-dashed border-primary flex flex-col items-center justify-center z-10 transition-opacity duration-200 pointer-events-none">
                <Download :size="32" class="text-primary mb-3 animate-bounce" />
                <div class="text-sm font-medium text-primary mb-1">释放以添加到编辑器</div>
                <div class="text-xs text-base-content/60">支持链接、文档、文字</div>
            </div>

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
        <div class="flex-1 overflow-hidden relative">
            <!-- 笔记列表拖拽遮罩 -->
            <div v-if="isDragging"
                class="absolute inset-0 bg-primary/20 border-2 border-dashed border-primary flex flex-col items-center justify-center z-10 transition-opacity duration-200 pointer-events-none">
                <Download :size="48" class="text-primary mb-4 animate-bounce" />
                <div class="text-base font-medium text-primary mb-2">释放以创建笔记</div>
                <div class="text-sm text-base-content/60">支持链接、文档（md、txt）、文字</div>
            </div>

            <div ref="noteListRef" class="p-3 h-full overflow-y-auto no-scrollbar" @scroll="handleNoteListScroll">
                <div v-if="notes.length === 0"
                    class="flex flex-col select-none items-center justify-center h-full text-base-content/40 text-center p-5">
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

        <!-- Toast 提示 -->
        <div
            :class="['toast toast-end z-[200] px-4 py-3 rounded-lg shadow-lg transition-all duration-300', toastVisible ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0', toastType === 'success' ? 'bg-success text-success-content' : toastType === 'error' ? 'bg-error text-error-content' : 'bg-info text-info-content']">
            {{ toastMessage }}
        </div>

        <!-- 处理中 Loading 提示 -->
        <div v-if="isProcessing" class="fixed bottom-6 right-6 z-[300] flex items-center gap-2 px-4 py-3 bg-base-100 border border-base-300 rounded-lg shadow-lg">
            <span class="loading loading-spinner loading-sm text-primary"></span>
            <span class="text-sm text-base-content/80">处理中...</span>
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
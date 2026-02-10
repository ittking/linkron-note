<script setup>
import { ref, onMounted, onBeforeUnmount, onActivated, nextTick, computed } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { Download, FileText, ChevronUp, Sidebar } from 'lucide-vue-next'
import NoteCard from '@/components/NoteCard.vue'
import NoteEditor from '@/components/NoteEditor.vue'
import NoteSidebar from '@/components/NoteSidebar.vue'
import Button from '@/components/ui/Button.vue'
import { useNoteStore } from '@/store/noteStore'
import { saveFile } from '@/utils/fileUpload'
import { extractTextFromFile, getFileTypeDescription } from '@/utils/textExtraction'
import { isSupportedFileType } from '@/utils/validator'
import { scrapeWebPage } from '@/utils/webScraper'
import { isValidUrl } from '@/utils/validator'
import { extractUrlFromUrlFile } from '@/utils/urlFileParser'
import { isUrlFile } from '@/utils/validator'
import { useWorkDirectory } from '@/composables/useWorkDirectory'
import { useToast } from '@/composables/useToast'
import { useConfirmDialog } from '@/composables/useConfirmDialog'

const noteStore = useNoteStore()

// 使用 composables
const { getWorkDirectory } = useWorkDirectory()
const { toastVisible, toastMessage, toastType, showToast } = useToast()
const { confirmVisible, confirmTitle, confirmContent, showConfirm, handleConfirmOk } = useConfirmDialog()

// 编辑器引用
const noteEditorRef = ref(null)
const editorContainerRef = ref(null)

// 滚动位置保存
const noteListRef = ref(null)
let savedScrollTop = 0
const isNoteListScrolledToTop = ref(true)

// 笔记展开状态管理和被切割笔记检测
const expandedNoteIds = ref(new Map())
const croppedNoteId = ref(null)

// NoteCard 组件引用管理
const noteCardRefs = ref(new Map())

function setNoteCardRef(noteId, ref) {
    if (ref) {
        noteCardRefs.value.set(noteId, ref)
    } else {
        noteCardRefs.value.delete(noteId)
    }
}

function collapseNote(noteId) {
    const noteCardRef = noteCardRefs.value.get(noteId)
    if (noteCardRef && noteCardRef.collapse) {
        noteCardRef.collapse()
    }
}

// 编辑器高度控制状态
const EDITOR_THRESHOLD = 150
let lastScrollTop = 0
let isScrollingDown = false

// 滚动优化状态
let rafId = null
let lastScrollTime = 0
const SCROLL_THROTTLE = 16 // ~60fps
let isScrolling = false
let scrollEndTimer = null

// 优化的编辑器高度更新逻辑
function updateEditorHeight(scrollTop) {
    isScrollingDown = scrollTop > lastScrollTop
    lastScrollTop = scrollTop

    if (isScrollingDown) {
        if (scrollTop > EDITOR_THRESHOLD && isNoteListScrolledToTop.value) {
            isNoteListScrolledToTop.value = false
        }
    } else {
        if (scrollTop < EDITOR_THRESHOLD / 2 && !isNoteListScrolledToTop.value) {
            isNoteListScrolledToTop.value = true
        }
    }
}

const notes = ref([])
const editorContent = ref('')
const isDragging = ref(false)
const isProcessing = ref(false)

// 分页相关状态
const currentPage = ref(1)
const pageSize = ref(20)
const hasMore = ref(true)
const isLoading = ref(false)

// 编辑相关状态
const editingNote = ref(null)
const isEditing = ref(false)
const shouldClearEditor = ref(false)

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
    setTimeout(() => {
        if (noteListRef.value && savedScrollTop > 0) {
            noteListRef.value.scrollTop = savedScrollTop
        }
    }, 50)
})

// 监听笔记列表滚动（优化版本）
function handleNoteListScroll() {
    if (!noteListRef.value) return

    const now = Date.now()
    const timeSinceLastScroll = now - lastScrollTime

    // 标记正在滚动
    isScrolling = true

    // 清除之前的定时器
    if (scrollEndTimer) {
        clearTimeout(scrollEndTimer)
    }

    // 设置滚动结束检测
    scrollEndTimer = setTimeout(() => {
        isScrolling = false
        // 滚动结束后再执行一次精确检测
        requestAnimationFrame(() => {
            detectCroppedNote()
        })
    }, 150)

    // 节流 + RAF 优化
    if (timeSinceLastScroll < SCROLL_THROTTLE) {
        if (rafId) return
        rafId = requestAnimationFrame(() => {
            rafId = null
        })
        lastScrollTime = now
        return
    }

    if (rafId) return

    rafId = requestAnimationFrame(() => {
        const scrollTop = noteListRef.value.scrollTop
        const { scrollHeight, clientHeight } = noteListRef.value
        const distanceToBottom = scrollHeight - scrollTop - clientHeight

        // 提前加载：200px 距离时触发
        if (distanceToBottom < 200 && hasMore.value && !isLoading.value) {
            loadNotes()
        }

        updateEditorHeight(scrollTop)

        // 滚动时简化检测，只检测可见区域附近的笔记
        detectCroppedNoteOptimized()

        lastScrollTime = now
        rafId = null
    })
}

// 优化的被切割笔记检测（滚动时使用，减少查询范围）
function detectCroppedNoteOptimized() {
    if (!noteListRef.value) return

    const containerRect = noteListRef.value.getBoundingClientRect()
    const containerBottom = containerRect.bottom

    // 只查询可见区域的元素，使用 for 循环比 forEach 更快
    const noteElements = noteListRef.value.querySelectorAll('[data-note-id]')
    let croppedId = null

    for (let i = 0; i < noteElements.length; i++) {
        const element = noteElements[i]
        const elementRect = element.getBoundingClientRect()
        const elementTop = elementRect.top
        const elementBottom = elementRect.bottom

        if (elementTop < containerBottom && elementBottom > containerBottom) {
            croppedId = element.getAttribute('data-note-id')
            break
        }
    }

    croppedNoteId.value = croppedId
}

// 原始检测方法（保留用于精确场景）
function detectCroppedNote() {
    if (!noteListRef.value) return

    const containerRect = noteListRef.value.getBoundingClientRect()
    const containerBottom = containerRect.bottom

    const noteElements = noteListRef.value.querySelectorAll('[data-note-id]')
    let croppedId = null

    noteElements.forEach(element => {
        const noteId = element.getAttribute('data-note-id')
        const elementRect = element.getBoundingClientRect()
        const elementTop = elementRect.top
        const elementBottom = elementRect.bottom

        if (elementTop < containerBottom && elementBottom > containerBottom) {
            croppedId = noteId
        }
    })

    croppedNoteId.value = croppedId
}

// 初始化
onMounted(async () => {
    await loadNotes()
    await nextTick()
    detectCroppedNote()
})

// 清理定时器和 RAF
onBeforeUnmount(() => {
    if (scrollEndTimer) {
        clearTimeout(scrollEndTimer)
    }
    if (rafId) {
        cancelAnimationFrame(rafId)
    }
})

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

        await nextTick()
        detectCroppedNote()
    } catch (error) {
        showToast('加载笔记失败', 'error')
    } finally {
        isLoading.value = false
    }
}

// 编辑器提交
async function handleEditorSubmit(noteData) {
    const content = noteData?.content || editorContent.value
    const images = noteData?.images || []

    if (content.trim() || images.length > 0) {
        if (isEditing.value && editingNote.value) {
            try {
                await noteStore.updateNote(editingNote.value.id, {
                    content: content,
                    images: images
                })
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
                showToast('笔记更新失败：' + error.message, 'error')
            }
        } else {
            const newNote = await noteStore.addNote({
                type: 'text',
                content: content,
                images: images
            })
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

    dragCounter = 0
    isDragging.value = false

    const dropInEditor = isDropInEditor(e.clientX, e.clientY)

    const files = e.dataTransfer.files
    if (files && files.length > 0) {
        for (let i = 0; i < files.length; i++) {
            if (dropInEditor) {
                handleFileToEditor(files[i])
            } else {
                handleDroppedFile(files[i])
            }
        }
        return
    }

    // 尝试获取不同类型的文本数据
    const uriListData = e.dataTransfer.getData('text/uri-list')
    const plainTextData = e.dataTransfer.getData('text/plain')
    const textData = uriListData || plainTextData

    if (textData) {
        if (dropInEditor) {
            handleDataToEditor(textData)
        } else {
            handleDroppedData(textData)
        }
    }
}

// 处理文件的公共逻辑
async function processFile(file, target = 'note') {
    const workDirectory = await getWorkDirectory()

    if (isUrlFile(file.name)) {
        // 处理 .url 文件
        try {
            isProcessing.value = true
            const url = await extractUrlFromUrlFile(file)
            if (url && isValidUrl(url)) {
                return { type: 'url', url: url }
            } else {
                showToast('无法从 .url 文件中提取有效的 URL', 'error')
                return null
            }
        } catch (error) {
            showToast('解析 .url 文件失败: ' + error.message, 'error')
            return null
        } finally {
            isProcessing.value = false
        }
    }

    if (!isSupportedFileType(file.name)) {
        showToast(`不支持的文件类型: ${file.name}`, 'error')
        return null
    }

    isProcessing.value = true
    try {
        const savedPath = await saveFile(file, 'file', workDirectory)
        const content = await extractTextFromFile(file, savedPath, workDirectory)
        return { content, savedPath }
    } catch (error) {
        showToast(`读取${getFileTypeDescription(file.name)}失败: ${error.message}`, 'error')
        return null
    } finally {
        isProcessing.value = false
    }
}

// 将文件添加到编辑器
async function handleFileToEditor(file) {
    const result = await processFile(file, 'editor')
    if (!result) return

    if (result.type === 'url') {
        // .url 文件提取的 URL，爬取内容到编辑器
        try {
            isProcessing.value = true
            const { content, images } = await scrapeWebPage(result.url)
            editorContent.value += (editorContent.value ? '<br>' : '') + content
            if (images && images.length > 0 && noteEditorRef.value?.addImages) {
                noteEditorRef.value.addImages(images)
            }
            showToast('网页内容已添加到编辑器', 'success')
        } catch (error) {
            showToast('网页抓取失败: ' + error.message, 'error')
        } finally {
            isProcessing.value = false
        }
        return
    }

    const htmlContent = result.content.replace(/\n/g, '<br>')
    editorContent.value += (editorContent.value ? '<br>' : '') + `<p>${htmlContent}</p>`
    showToast(`${getFileTypeDescription(file.name)}内容已添加到编辑器`, 'success')
}

// 将数据添加到编辑器
async function handleDataToEditor(data) {
    const isUrl = isValidUrl(data)

    if (isUrl) {
        try {
            isProcessing.value = true
            const { content, images } = await scrapeWebPage(data)
            if (content) {
                editorContent.value += (editorContent.value ? '<br>' : '') + content
                if (images && images.length > 0 && noteEditorRef.value?.addImages) {
                    noteEditorRef.value.addImages(images)
                }
                showToast('网页内容已添加到编辑器', 'success')
            } else {
                showToast('网页内容为空', 'error')
            }
        } catch (error) {
            showToast('网页抓取失败: ' + error.message, 'error')
        } finally {
            isProcessing.value = false
        }
    } else {
        editorContent.value += (editorContent.value ? '<br>' : '') + data
        showToast('文本已添加到编辑器', 'success')
    }
}

// 处理拖拽数据（创建笔记）
async function handleDroppedData(data) {
    const isUrl = isValidUrl(data)

    if (isUrl) {
        await createLinkNote(data)
    } else {
        await createTextNote(data)
    }
}

// 处理拖拽文件（创建笔记）
async function handleDroppedFile(file) {
    const result = await processFile(file, 'note')
    if (!result) return

    if (result.type === 'url') {
        await createLinkNote(result.url)
        return
    }

    const htmlContent = `<p>${result.content.replace(/\n/g, '<br>')}</p>`

    const newNote = await noteStore.addNote({
        type: 'file',
        content: htmlContent,
        extractUrl: result.savedPath
    })
    notes.value.unshift(newNote)
    showToast(`${getFileTypeDescription(file.name)}笔记创建成功`, 'success')
}

// 创建链接笔记
async function createLinkNote(url) {
    if (!isValidUrl(url)) {
        showToast('无效的 URL 格式', 'error')
        return
    }

    try {
        isProcessing.value = true
        const { content, images } = await scrapeWebPage(url)
        const newNote = await noteStore.addNote({
            type: 'link',
            content: content,
            sourceUrl: url,
            images: images || []
        })
        notes.value.unshift(newNote)
        showToast('链接笔记创建成功', 'success')
    } catch (error) {
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

// NoteCard 展开/收起事件处理
function handleNoteExpand(noteId) {
    expandedNoteIds.value.set(noteId, true)
    nextTick(() => {
        detectCroppedNote()
    })
}

function handleNoteCollapse(noteId) {
    expandedNoteIds.value.set(noteId, false)
    nextTick(() => {
        detectCroppedNote()
    })
}

// 判断是否显示浮动收起按钮
const shouldShowCollapseButton = computed(() => {
    const croppedId = croppedNoteId.value
    if (!croppedId || !expandedNoteIds.value) return false
    return expandedNoteIds.value.get(croppedId) === true
})

// 处理浮动收起按钮点击
function handleCollapseCroppedNote() {
    if (croppedNoteId.value) {
        collapseNote(croppedNoteId.value)
    }
}

// 菜单事件
function handleMenuEdit(note) {
    editingNote.value = note
    shouldClearEditor.value = false
    nextTick(() => {
        editorContent.value = note.content
    })
    isEditing.value = true
    showToast('进入编辑模式', 'info')
}

function handleMenuDelete(note) {
    showConfirm('确认删除', '确定要删除这条笔记吗？', async () => {
        await noteStore.deleteNote(note.id)
        notes.value = notes.value.filter(n => n.id !== note.id)
        showToast('笔记已删除', 'success')
    })
}

// 取消编辑
function handleCancelEdit() {
    shouldClearEditor.value = true
    if (noteEditorRef.value?.clearDeletedImages) {
        noteEditorRef.value.clearDeletedImages()
    }
    editingNote.value = null
    isEditing.value = false
    editorContent.value = ''
    showToast('已取消编辑', 'info')
}

// 计算显示的笔记列表
const displayNotes = computed(() => {
    return notes.value
})

// 侧边栏状态
const isSidebarOpen = ref(false)

// 打开侧边栏
function openSidebar() {
    isSidebarOpen.value = true
}

// 关闭侧边栏
function closeSidebar() {
    isSidebarOpen.value = false
}

// 选择标签
function handleSelectTag(tagPath) {
    console.log('选择标签:', tagPath)
    // TODO: 根据标签筛选笔记
    closeSidebar()
}
</script>

<template>
    <div class="h-full flex" @dragenter="handleDragEnter" @dragleave="handleDragLeave"
        @dragover="handleDragOver" @drop="handleDrop">
        <!-- 主内容 -->
        <div class="flex-1 flex flex-col max-w-200 mx-auto">

            <!-- 编辑器区域 -->
            <div ref="editorContainerRef" class="px-4 py-3 relative">
            <!-- 编辑器拖拽遮罩 -->
            <div v-if="isDragging"
                class="absolute inset-0 bg-primary/2 border-2 border-dashed border-primary flex flex-col items-center justify-center z-10 transition-opacity duration-200 pointer-events-none">
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
                class="absolute inset-0 bg-primary/2 border-2 border-dashed border-primary flex flex-col items-center justify-center z-10 transition-opacity duration-200 pointer-events-none">
                <Download :size="48" class="text-primary mb-4 animate-bounce" />
                <div class="text-base font-medium text-primary mb-2">释放以创建笔记</div>
                <div class="text-sm text-base-content/60">支持链接、文档（md、txt）、文字</div>
            </div>

            <div ref="noteListRef" class="p-3 h-full overflow-y-auto no-scrollbar relative" @scroll="handleNoteListScroll">
                <div v-if="notes.length === 0"
                    class="flex flex-col select-none items-center justify-center h-full text-base-content/40 text-center p-5">
                    <FileText :size="64" class="mb-4 opacity-50" />
                    <div class="text-base font-medium mb-2 text-base-content/60">暂无笔记</div>
                    <div class="text-sm leading-relaxed max-w-[240px]">拖拽链接或文字到这里创建笔记</div>
                </div>

                <NoteCard v-for="note in displayNotes" :key="note.id" :ref="(ref) => setNoteCardRef(note.id, ref)" :note="note"
                    @click="handleCardClick" @edit="handleMenuEdit" @delete="handleMenuDelete"
                    @expand="handleNoteExpand" @collapse="handleNoteCollapse" />

                <!-- Loading 组件 -->
                <div v-if="isLoading" class="flex justify-center py-4">
                    <span class="loading loading-spinner text-primary"></span>
                </div>

                <!-- 浮动收起按钮 -->
                <button v-if="shouldShowCollapseButton" @click="handleCollapseCroppedNote"
                    class="fixed bottom-4 left-1/2 -translate-x-1/2 z-50 bg-base-100 border border-base-200 shadow-lg px-4 py-2 rounded-lg text-xs text-primary hover:text-primary/80 flex items-center gap-1 transition-all duration-200">
                    收起笔记
                    <ChevronUp :size="14" />
                </button>
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
                    <Button variant="ghost" size="sm" @click="confirmVisible = false">取消</Button>
                    <Button variant="error" size="sm" @click="handleConfirmOk">删除</Button>
                </div>
            </div>
            <form method="dialog" class="modal-backdrop bg-black/50">
                <button @click="confirmVisible = false"></button>
            </form>
        </dialog>
        </div>

        <!-- 侧边栏悬浮图标 -->
        <button @click="openSidebar"
            class="fixed bottom-6 right-6 z-50 w-12 h-12 bg-primary text-primary-content rounded-full flex items-center justify-center shadow-lg hover:bg-primary/90 hover:scale-105 transition-all duration-200"
            title="打开标签侧边栏">
            <Sidebar :size="20" />
        </button>

        <!-- 侧边栏 -->
        <NoteSidebar :is-open="isSidebarOpen" @close="closeSidebar" @select-tag="handleSelectTag" />
    </div>
</template>
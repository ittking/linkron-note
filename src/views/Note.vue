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
import { useSettingStore } from '@/store/settingStore'
import { optimizeWebContent } from '@/utils/aiOptimizer'

const noteStore = useNoteStore()
const settingStore = useSettingStore()

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

function setNoteCardRef(noteId, cardRef) {
    if (cardRef) {
        noteCardRefs.value.set(noteId, cardRef)
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
const processingCount = ref(0) // 处理计数器

// 工作目录缓存
let cachedWorkDirectory = null

// 开始处理
function startProcessing() {
    processingCount.value++
    isProcessing.value = true
}

// 结束处理
function endProcessing() {
    processingCount.value--
    if (processingCount.value <= 0) {
        processingCount.value = 0
        isProcessing.value = false
    }
}

// 获取缓存的工作目录
async function getCachedWorkDirectory() {
    if (!cachedWorkDirectory) {
        cachedWorkDirectory = await getWorkDirectory()
    }
    return cachedWorkDirectory
}

// 标签筛选相关状态
const selectedTags = ref([])
const filteredNoteCount = ref(0)
const isTagFilterMode = ref(false)

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

        // 滚动时使用优化检测
        detectCroppedNote(true)

        lastScrollTime = now
        rafId = null
    })
}

// 检测被切割的笔记
function detectCroppedNote(optimized = false) {
    if (!noteListRef.value) return

    const containerRect = noteListRef.value.getBoundingClientRect()
    const containerBottom = containerRect.bottom

    const noteElements = noteListRef.value.querySelectorAll('[data-note-id]')
    let croppedId = null

    if (optimized) {
        // 优化版本：使用 for 循环比 forEach 更快
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
    } else {
        // 标准版本：用于精确场景
        noteElements.forEach(element => {
            if (croppedId) return // 已找到，提前退出
            const noteId = element.getAttribute('data-note-id')
            const elementRect = element.getBoundingClientRect()
            const elementTop = elementRect.top
            const elementBottom = elementRect.bottom

            if (elementTop < containerBottom && elementBottom > containerBottom) {
                croppedId = noteId
            }
        })
    }

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
    if (isLoading.value || isTagFilterMode.value) return

    if (reset) {
        currentPage.value = 1
        hasMore.value = true
    }

    if (!hasMore.value) return

    isLoading.value = true

    try {
        const result = await noteStore.getNotes(currentPage.value, pageSize.value)
        const newNotes = result.notes || []
        const total = result.total || 0

        // 使用 total 字段判断是否还有更多数据
        hasMore.value = notes.value.length + newNotes.length < total

        if (reset) {
            notes.value = newNotes
        } else {
            notes.value = [...notes.value, ...newNotes]
        }

        currentPage.value++

        await nextTick()
        detectCroppedNote()
    } catch (error) {
        showToast('加载笔记失败: ' + error.message, 'error')
        // 加载失败时，假设没有更多数据，避免频繁重试
        hasMore.value = false
    } finally {
        isLoading.value = false
    }
}

// 编辑器提交
async function handleEditorSubmit(noteData) {
    const content = noteData?.content || editorContent.value

    if (content.trim()) {
        if (isEditing.value && editingNote.value) {
            try {
                await noteStore.updateNote(editingNote.value.id, {
                    content: content
                })
                const index = notes.value.findIndex(n => n.id === editingNote.value.id)
                if (index !== -1) {
                    notes.value[index] = {
                        ...notes.value[index],
                        content: content
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
                content: content
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
    const workDirectory = await getCachedWorkDirectory()

    if (isUrlFile(file.name)) {
        // 处理 .url 文件
        try {
            startProcessing()
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
            endProcessing()
        }
    }

    if (!isSupportedFileType(file.name)) {
        showToast(`不支持的文件类型: ${file.name}`, 'error')
        return null
    }

    startProcessing()
    try {
        const savedPath = await saveFile(file, 'file', workDirectory)
        const content = await extractTextFromFile(file, savedPath, workDirectory)
        return { content, savedPath }
    } catch (error) {
        showToast(`读取${getFileTypeDescription(file.name)}失败: ${error.message}`, 'error')
        return null
    } finally {
        endProcessing()
    }
}

// 将文件添加到编辑器
async function handleFileToEditor(file) {
    const result = await processFile(file, 'editor')
    if (!result) return

    if (result.type === 'url') {
        // .url 文件提取的 URL，爬取内容到编辑器
        try {
            startProcessing()

            // 第一步：获取网页内容
            const { content, images } = await scrapeWebPage(result.url)
            if (!content) {
                showToast('网页内容为空', 'error')
                return
            }

            // 第二步：调用 AI 优化内容
            let finalContent = content
            try {
                const { content: optimizedContent, optimized } = await optimizeWebContent(result.url, content)
                finalContent = optimizedContent

                if (optimized) {
                    showToast('AI 文章生成成功', 'success')
                }
            } catch (error) {
                showToast('AI 优化失败，使用原始内容: ' + error.message, 'error')
            }

            // 添加到编辑器
            editorContent.value += (editorContent.value ? '<br>' : '') + finalContent
        } catch (error) {
            showToast('网页抓取失败: ' + error.message, 'error')
        } finally {
            endProcessing()
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
            startProcessing()

            // 第一步：获取网页内容
            const { content } = await scrapeWebPage(data)
            if (!content) {
                showToast('网页内容为空', 'error')
                return
            }

            // 第二步：调用 AI 优化内容
            let finalContent = content
            try {
                const { content: optimizedContent, optimized } = await optimizeWebContent(data, content)
                finalContent = optimizedContent

                if (optimized) {
                    showToast('AI 文章生成成功', 'success')
                }
            } catch (error) {
                showToast('AI 优化失败，使用原始内容: ' + error.message, 'error')
            }

            // 添加到编辑器
            editorContent.value += (editorContent.value ? '<br>' : '') + finalContent
        } catch (error) {
            showToast('网页抓取失败: ' + error.message, 'error')
        } finally {
            endProcessing()
        }
        return
    }

    // 普通文本直接添加到编辑器
    editorContent.value += (editorContent.value ? '<br>' : '') + `<p>${data}</p>`
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
        startProcessing()

        // 第一步：获取网页内容
        const { content } = await scrapeWebPage(url)

        // 第二步：调用 AI 优化内容
        let finalContent = content
        try {
            const { content: optimizedContent, optimized } = await optimizeWebContent(url, content)
            finalContent = optimizedContent

            if (optimized) {
                showToast('AI 文章生成成功', 'success')
            } else {
                showToast('链接笔记创建成功', 'success')
            }
        } catch (error) {
            console.error('AI 优化失败:', error)
            showToast('AI 优化失败，使用原始内容: ' + error.message, 'error')
            finalContent = content
        }

        const newNote = await noteStore.addNote({
            type: 'link',
            content: finalContent,
            sourceUrl: url
        })
        notes.value.unshift(newNote)
    } catch (error) {
        showToast('链接抓取失败: ' + error.message, 'error')
    } finally {
        endProcessing()
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

function handleMenuPin(note) {
    noteStore.pinNote(note.id).then(() => {
        // 更新笔记的 pinned 状态
        const index = notes.value.findIndex(n => n.id === note.id)
        if (index !== -1) {
            notes.value[index] = {
                ...notes.value[index],
                pinned: !note.pinned
            }
        }
        // 重新加载笔记列表以应用正确的排序
        loadNotes(true)
        showToast(note.pinned ? '已取消置顶' : '已置顶', 'success')
    }).catch(error => {
        showToast('操作失败：' + error.message, 'error')
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
    // 验证标签路径有效
    if (!tagPath || typeof tagPath !== 'string') {
        closeSidebar()
        return
    }
    // 添加标签到筛选列表
    if (!selectedTags.value.includes(tagPath)) {
        selectedTags.value.push(tagPath)
        isTagFilterMode.value = true
        filterNotesByTags()
    }
    closeSidebar()
}

// 移除标签
function removeTag(tagPath) {
    selectedTags.value = selectedTags.value.filter(t => t !== tagPath)
    if (selectedTags.value.length === 0) {
        isTagFilterMode.value = false
        loadNotes(true)
    } else {
        filterNotesByTags()
    }
}

// 清空所有标签
function clearAllTags() {
    selectedTags.value = []
    isTagFilterMode.value = false
    loadNotes(true)
}

// 根据标签筛选笔记
async function filterNotesByTags() {
    isLoading.value = true
    try {
        const workDirectory = await getCachedWorkDirectory()
        const [filteredNotes, count] = await Promise.all([
            invoke('get_notes_by_tags', { tags: selectedTags.value, workDirectory }),
            invoke('count_notes_by_tags', { tags: selectedTags.value, workDirectory })
        ])
        // 新数据到达后才更新 notes，避免闪烁
        await nextTick()
        notes.value = filteredNotes
        filteredNoteCount.value = count
    } catch (error) {
        showToast('筛选笔记失败：' + error.message, 'error')
    } finally {
        isLoading.value = false
    }
}
</script>

<template>
    <div class="h-full pb-2" @dragenter="handleDragEnter" @dragleave="handleDragLeave" @dragover="handleDragOver"
        @drop="handleDrop">
        <!-- 主内容 -->
        <div class="h-full flex flex-col max-w-200 mx-auto w-full">

            <!-- 编辑器区域 -->
            <div ref="editorContainerRef" class="p-3 relative">
                <!-- 编辑器拖拽遮罩 -->
                <div v-if="isDragging"
                    class="absolute inset-0 bg-primary/2 border-2 border-dashed border-primary flex flex-col items-center justify-center z-10 transition-opacity duration-200 pointer-events-none">
                    <Download :size="32" class="text-primary mb-3 animate-bounce" />
                    <div class="text-sm font-medium text-primary mb-1">释放以添加到编辑器</div>
                    <div class="text-xs text-base-content/60">支持链接、文档、文字</div>
                </div>

                <NoteEditor ref="noteEditorRef" v-model="editorContent" placeholder="现在的想法是..."
                    :is-scrolled-to-top="isNoteListScrolledToTop" :is-editing="isEditing"
                    :should-clear="shouldClearEditor" @submit="handleEditorSubmit">
                    <template #actions>
                        <button v-if="isEditing" @click="handleCancelEdit"
                            class="px-3 h-7 rounded-md flex items-center justify-center transition-all duration-200 bg-base-300 text-base-content/60 hover:bg-base-200 hover:text-base-content text-xs"
                            title="取消编辑">
                            取消
                        </button>
                    </template>
                </NoteEditor>
            </div>

            <!-- 标签筛选栏 -->
            <div v-if="isTagFilterMode" class="px-4 py-3 border-t border-base-200 bg-base-100">
                <div class="flex items-center justify-between gap-2">
                    <!-- 左侧：标签列表 -->
                    <div class="flex items-center gap-2 flex-1 flex-wrap">
                        <div class="flex flex-wrap items-center gap-1">
                            <div v-for="tag in selectedTags" :key="tag"
                                class="inline-flex items-center gap-1 px-2 py-1 bg-primary/10 text-primary rounded-md text-xs hover:bg-primary/20 transition-colors cursor-pointer group whitespace-normal"
                                @click="removeTag(tag)">
                                <span>#{{ tag }}</span>
                                <button class="opacity-60 group-hover:opacity-100 transition-opacity">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"
                                        fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
                                        stroke-linejoin="round">
                                        <line x1="18" y1="6" x2="6" y2="18"></line>
                                        <line x1="6" y1="6" x2="18" y2="18"></line>
                                    </svg>
                                </button>
                            </div>
                        </div>
                    </div>

                    <!-- 右侧：操作和数量 -->
                    <div class="flex items-center gap-3 flex-shrink-0">
                        <button v-if="selectedTags.length > 1" @click="clearAllTags"
                            class="text-xs text-base-content/40 hover:text-base-content/60 transition-colors">
                            清空
                        </button>
                        <span class="text-xs text-base-content/60">共 {{ filteredNoteCount }} 条笔记</span>
                    </div>
                </div>
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

                <div ref="noteListRef" class="p-3 pt-0 h-full overflow-y-auto no-scrollbar relative"
                    @scroll="handleNoteListScroll">
                    <div v-if="notes.length === 0"
                        class="flex flex-col select-none items-center justify-center h-full text-base-content/40 text-center p-5">
                        <FileText :size="64" class="mb-4 opacity-50" />
                        <div class="text-base font-medium mb-2 text-base-content/60">暂无笔记</div>
                        <div class="text-sm leading-relaxed max-w-[240px]">拖拽链接或文字到这里创建笔记</div>
                    </div>

                    <NoteCard v-for="note in notes" :key="note.id" :ref="(ref) => setNoteCardRef(note.id, ref)"
                        :note="note" @edit="handleMenuEdit" @delete="handleMenuDelete" @pin="handleMenuPin"
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
            <div v-if="isProcessing"
                class="fixed bottom-6 right-6 z-[300] flex items-center gap-2 px-4 py-3 bg-base-100 border border-base-300 rounded-lg shadow-lg">
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
            class="fixed bottom-6 right-6 z-50 w-10 h-10 bg-primary text-primary-content rounded-full flex items-center justify-center shadow-lg hover:bg-primary/90 hover:scale-105 transition-all duration-200"
            title="打开标签侧边栏">
            <Sidebar :size="18" />
        </button>

        <!-- 侧边栏 -->
        <NoteSidebar :is-open="isSidebarOpen" @close="closeSidebar" @select-tag="handleSelectTag" />
    </div>
</template>
<script setup>
import { ref, onMounted, onActivated, nextTick } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { Download } from 'lucide-vue-next'
import NoteCard from '@/components/NoteCard.vue'
import NoteEditor from '@/components/NoteEditor.vue'
import { useNoteStore } from '@/store/noteStore'
import { saveFile, getResourceUrl } from '@/utils/fileUpload'
import { extractTextFromFile, isSupportedFileType, getFileTypeDescription } from '@/utils/textExtraction'
import { scrapeWebPage, isValidUrl, formatWebPageToNote } from '@/utils/webScraper'

const noteStore = useNoteStore()

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
const isDragging = ref(false)
const dragCounter = ref(0) // 拖拽计数器
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

// 标签筛选相关状态
const currentFilterTag = ref(null)
const isFiltering = ref(false)

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
async function handleEditorSubmit() {
    if (editorContent.value.trim()) {
        if (isEditing.value && editingNote.value) {
            // 编辑模式：更新笔记
            try {
                await noteStore.updateNote(editingNote.value.id, {
                    content: editorContent.value
                })
                // 直接在数组中更新笔记
                const index = notes.value.findIndex(n => n.id === editingNote.value.id)
                if (index !== -1) {
                    notes.value[index] = {
                        ...notes.value[index],
                        content: editorContent.value
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
                content: editorContent.value
            })
            // 直接在数组开头添加新笔记
            notes.value.unshift(newNote)
            editorContent.value = ''
            showToast('笔记创建成功', 'success')
        }
    }
}

// 图片上传处理 - 已废弃，图片现在通过编辑器插入，不单独创建笔记
async function handleImageUpload(imagePath) {
    // 这个函数不再使用，因为图片现在是通过编辑器插入的
    // 用户提交编辑器内容时会一并提交图片
    console.log('handleImageUpload 已废弃，图片应通过编辑器提交')
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
    e.stopPropagation() // 阻止事件冒泡，防止触发多次
    isDragging.value = false
    dragCounter.value = 0

    // 优先处理文件
    const files = e.dataTransfer.files
    if (files && files.length > 0) {
        for (let i = 0; i < files.length; i++) {
            handleDroppedFile(files[i])
        }
        return
    }

    // 处理 URL/文本数据（只有在没有文件时才处理）
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

    // 图片文件
    if (file.type.startsWith('image/')) {
        try {
            // 保存图片文件
            const imagePath = await saveFile(file, 'image', workDirectory)
            const resourceUrl = await getResourceUrl(imagePath)
            
            // 创建带图片的 HTML 内容
            const imageHtml = `<p><img src="${resourceUrl}" alt="${file.name}" style="max-width: 100%;"></p>`
            
            // 创建图文笔记
            const newNote = await noteStore.addNote({
                type: 'text',
                content: imageHtml
            })
            notes.value.unshift(newNote)
            showToast('图片已添加到笔记', 'success')
        } catch (error) {
            console.error('图片保存失败:', error)
            showToast('图片保存失败', 'error')
        }
    }
    // .url 文件 - 解析文件内容获取 URL
    else if (file.name.endsWith('.url')) {
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
            
            // 提取文本内容（直接从文件读取，不保存）
            const content = await extractTextFromFile(file, null, workDirectory)
            
            // 创建图文笔记（不包含文件名）
            const htmlContent = `<p>${content.replace(/\n/g, '<br>')}</p>`
            
            const newNote = await noteStore.addNote({
                type: 'text',
                content: htmlContent
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

// 从 .url 文件中提取 URL
async function extractUrlFromUrlFile(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader()
        
        reader.onload = (e) => {
            try {
                const content = e.target.result
                
                // .url 文件格式通常是 INI 格式
                // 查找 URL= 这一行
                const urlMatch = content.match(/^URL=(.+)$/m)
                
                if (urlMatch && urlMatch[1]) {
                    resolve(urlMatch[1].trim())
                } else {
                    // 尝试其他格式：InternetShortcut
                    const internetShortcutMatch = content.match(/^\[InternetShortcut\]([\s\S]*?)^\[.*\]$/m)
                    if (internetShortcutMatch) {
                        const sectionContent = internetShortcutMatch[1]
                        const urlLine = sectionContent.match(/^URL=(.+)$/m)
                        if (urlLine && urlLine[1]) {
                            resolve(urlLine[1].trim())
                        }
                    }
                    
                    // 如果都找不到，尝试查找任何包含 http/https 的行
                    const httpMatch = content.match(/(https?:\/\/[^\s]+)/)
                    if (httpMatch) {
                        resolve(httpMatch[1].trim())
                    } else {
                        reject(new Error('未找到 URL'))
                    }
                }
            } catch (error) {
                reject(error)
            }
        }
        
        reader.onerror = () => {
            reject(new Error('读取文件失败'))
        }
        
        reader.readAsText(file)
    })
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
        
        // 创建链接笔记
        const newNote = await noteStore.addNote({
            type: 'link',
            content: content,
            sourceUrl: url
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
        // 直接从当前数组中移除笔记，避免重新加载导致的空白
        notes.value = notes.value.filter(n => n.id !== note.id)
        showToast('笔记已删除', 'success')
    }
    confirmVisible.value = true
}

// 标签点击事件
function handleTagClick(tag) {
    loadNotesByTag(tag.id)
}

// 按标签加载笔记
async function loadNotesByTag(tagId) {
    isFiltering.value = true
    currentFilterTag.value = tagId
    
    try {
        const newNotes = await noteStore.getNotesByTag(tagId, 1, 20)
        notes.value = newNotes
    } catch (error) {
        console.error('Failed to load notes by tag:', error)
        showToast('加载笔记失败', 'error')
    }
}

// 清除筛选
function clearFilter() {
    isFiltering.value = false
    currentFilterTag.value = null
    loadNotes(true)
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
            <NoteEditor v-model="editorContent" :placeholder="isEditing ? '编辑笔记...' : '现在的想法是...'"
                :is-scrolled-to-top="isNoteListScrolledToTop" :is-editing="isEditing" @submit="handleEditorSubmit">
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
            <!-- 筛选状态显示 -->
            <div v-if="isFiltering" class="px-4 py-2 bg-base-200 border-b border-base-300">
                <div class="flex items-center gap-2 text-sm">
                    <span class="text-base-content/60">筛选标签：</span>
                    <span class="text-primary font-medium">#{{ currentFilterTag }}</span>
                    <button @click="clearFilter"
                        class="text-xs text-base-content/60 hover:text-base-content underline ml-2">
                        清除筛选
                    </button>
                </div>
            </div>

            <div ref="noteListRef" class="p-3 h-full overflow-y-auto no-scrollbar" @scroll="handleNoteListScroll">
                <div v-if="notes.length === 0"
                    class="flex flex-col items-center justify-center h-full text-base-content/40 text-center p-5">
                    <div class="text-5xl mb-4 opacity-50">📝</div>
                    <div class="text-base font-medium mb-2 text-base-content/60">暂无笔记</div>
                    <div class="text-sm leading-relaxed max-w-[240px]">拖拽链接、文字或图片到这里创建笔记</div>
                </div>

                <NoteCard
                    v-for="note in notes"
                    :key="note.id"
                    :note="note"
                    @click="handleCardClick"
                    @open="handleMenuOpen"
                    @edit="handleMenuEdit"
                    @delete="handleMenuDelete"
                    @tag-click="handleTagClick"
                />

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
            <div class="text-sm text-base-content/60">支持链接、文字、图片</div>
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
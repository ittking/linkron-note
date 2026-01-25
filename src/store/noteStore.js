import { Store } from '@tauri-apps/plugin-store'
import { ulid } from 'ulid'
import { useSettingStore } from './settingStore'

/**
 * 笔记数据持久化存储
 * 使用 Tauri Store 插件实现笔记的增删改查功能
 */
export function useNoteStore() {
  let store = null
  const settingStore = useSettingStore()

  /**
   * 初始化存储
   */
  async function initStore() {
    if (!store) {
      // 获取工作目录配置
      const workDirectory = await settingStore.get('workDirectory', '')
      
      // 如果有工作目录，则使用完整路径；否则使用默认路径
      const storePath = workDirectory ? `${workDirectory}/notes.json` : 'notes.json'
      
      store = await Store.load(storePath)
    }
    return store
  }

  /**
   * 生成唯一 ID
   */
  function generateId() {
    return ulid()
  }

  /**
   * 获取所有笔记
   */
  async function getNotes() {
    await initStore()
    const notes = await store.get('notes') || []
    // 按更新时间倒序排列
    return notes.sort((a, b) => new Date(b.updatedAt) - new Date(a.updatedAt))
  }

  /**
   * 获取单个笔记
   */
  async function getNote(id) {
    await initStore()
    const notes = await store.get('notes') || []
    return notes.find(note => note.id === id)
  }

  /**
   * 添加笔记
   */
  async function addNote(noteData) {
    await initStore()
    const notes = await store.get('notes') || []
    
    const newNote = {
      id: generateId(),
      type: noteData.type || 'text',
      title: noteData.title || '未命名笔记',
      content: noteData.content || '',
      sourceUrl: noteData.sourceUrl || '',
      images: noteData.images || [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    }
    
    notes.unshift(newNote)
    await store.set('notes', notes)
    await store.save()
    
    return newNote
  }

  /**
   * 更新笔记
   */
  async function updateNote(id, updates) {
    await initStore()
    const notes = await store.get('notes') || []
    const index = notes.findIndex(note => note.id === id)
    
    if (index !== -1) {
      notes[index] = {
        ...notes[index],
        ...updates,
        updatedAt: new Date().toISOString()
      }
      await store.set('notes', notes)
      await store.save()
      return notes[index]
    }
    
    return null
  }

  /**
   * 删除笔记
   */
  async function deleteNote(id) {
    await initStore()
    const notes = await store.get('notes') || []
    const filteredNotes = notes.filter(note => note.id !== id)
    
    await store.set('notes', filteredNotes)
    await store.save()
    
    return filteredNotes
  }

  /**
   * 搜索笔记
   */
  async function searchNotes(keyword) {
    await initStore()
    const notes = await store.get('notes') || []
    
    if (!keyword || keyword.trim() === '') {
      return notes.sort((a, b) => new Date(b.updatedAt) - new Date(a.updatedAt))
    }
    
    const searchLower = keyword.toLowerCase()
    const filteredNotes = notes.filter(note => {
      return note.title.toLowerCase().includes(searchLower) ||
             note.content.toLowerCase().includes(searchLower)
    })
    
    return filteredNotes.sort((a, b) => new Date(b.updatedAt) - new Date(a.updatedAt))
  }

  /**
   * 清空所有笔记
   */
  async function clearNotes() {
    await initStore()
    await store.set('notes', [])
    await store.save()
  }

  return {
    getNotes,
    getNote,
    addNote,
    updateNote,
    deleteNote,
    searchNotes,
    clearNotes
  }
}
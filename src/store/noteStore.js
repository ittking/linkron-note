import { invoke } from '@tauri-apps/api/core'
import { useSettingStore } from './settingStore'

/**
 * 笔记数据持久化存储
 * 使用 SQLite 数据库实现笔记的增删改查功能
 */
export function useNoteStore() {
  const settingStore = useSettingStore()

  /**
   * 获取工作目录
   */
  async function getWorkDirectory() {
    return await settingStore.get('workDirectory', '')
  }

  /**
   * 初始化数据库
   */
  async function initDatabase() {
    const workDirectory = await getWorkDirectory()
    await invoke('init_database', { workDirectory })
  }

  /**
   * 获取所有笔记（分页）
   */
  async function getNotes(page = 1, pageSize = 20) {
    const workDirectory = await getWorkDirectory()
    return await invoke('get_all_notes', { page, pageSize, workDirectory })
  }

  /**
   * 获取笔记总数
   */
  async function getNotesCount() {
    const workDirectory = await getWorkDirectory()
    return await invoke('get_notes_count', { workDirectory })
  }

  /**
   * 获取单个笔记
   */
  async function getNote(id) {
    const workDirectory = await getWorkDirectory()
    return await invoke('get_note', { id, workDirectory })
  }

  /**
   * 添加笔记
   */
  async function addNote(noteData) {
    const workDirectory = await getWorkDirectory()
    const noteDataWithDefaults = {
      type: noteData.type || 'text',
      content: noteData.content || '',
      sourceUrl: noteData.sourceUrl || null,
      images: noteData.images || []
    }
    return await invoke('create_note', { 
      noteData: noteDataWithDefaults, 
      workDirectory 
    })
  }

  /**
   * 更新笔记
   */
  async function updateNote(id, updates) {
    const workDirectory = await getWorkDirectory()
    return await invoke('update_note', { id, updates, workDirectory })
  }

  /**
   * 删除笔记
   */
  async function deleteNote(id) {
    const workDirectory = await getWorkDirectory()
    await invoke('delete_note', { id, workDirectory })
  }

  /**
   * 搜索笔记
   */
  async function searchNotes(keyword) {
    const workDirectory = await getWorkDirectory()
    
    if (!keyword || keyword.trim() === '') {
      return getNotes()
    }
    
    return await invoke('search_notes', { keyword, workDirectory })
  }

  /**
   * 清空所有笔记
   */
  async function clearNotes() {
    const notes = await getNotes()
    for (const note of notes) {
      await deleteNote(note.id)
    }
  }

  /**
   * 从 JSON 迁移到 SQLite
   */
  async function migrateFromJson() {
    const workDirectory = await getWorkDirectory()
    return await invoke('migrate_from_json', { workDirectory })
  }

  return {
    initDatabase,
    getNotes,
    getNotesCount,
    getNote,
    addNote,
    updateNote,
    deleteNote,
    searchNotes,
    clearNotes,
    migrateFromJson
  }
}
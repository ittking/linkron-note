import { Store } from '@tauri-apps/plugin-store'

/**
 * 设置存储 Store
 * 专门用于应用设置的持久化存储
 */
export function useSettingStore() {
  let store = null

  /**
   * 初始化存储
   */
  async function initStore() {
    if (!store) {
      store = await Store.load('settings.json')
    }
    return store
  }

  /**
   * 获取值
   * @param {string} key - 键名
   * @param {any} defaultValue - 默认值
   * @returns {Promise<any>}
   */
  async function get(key, defaultValue = null) {
    await initStore()
    return await store.get(key, defaultValue)
  }

  /**
   * 设置值
   * @param {string} key - 键名
   * @param {any} value - 值
   * @returns {Promise<void>}
   */
  async function set(key, value) {
    await initStore()
    await store.set(key, value)
    await store.save()
  }

  /**
   * 删除值
   * @param {string} key - 键名
   * @returns {Promise<void>}
   */
  async function remove(key) {
    await initStore()
    await store.delete(key)
    await store.save()
  }

  /**
   * 清空所有数据
   * @returns {Promise<void>}
   */
  async function clear() {
    await initStore()
    await store.clear()
    await store.save()
  }

  /**
   * 检查键是否存在
   * @param {string} key - 键名
   * @returns {Promise<boolean>}
   */
  async function has(key) {
    await initStore()
    return await store.has(key)
  }

  /**
   * 获取所有键
   * @returns {Promise<string[]>}
   */
  async function keys() {
    await initStore()
    return await store.keys()
  }

  /**
   * 获取所有值
   * @returns {Promise<any[]>}
   */
  async function values() {
    await initStore()
    return await store.values()
  }

  /**
   * 获取所有键值对
   * @returns {Promise<Array<[string, any]>>}
   */
  async function entries() {
    await initStore()
    return await store.entries()
  }

  /**
   * 获取存储长度
   * @returns {Promise<number>}
   */
  async function length() {
    await initStore()
    return await store.length()
  }

  return {
    get,
    set,
    remove,
    clear,
    has,
    keys,
    values,
    entries,
    length,
  }
}
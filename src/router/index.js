import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    redirect: '/note'
  },
  {
    path: '/note',
    name: 'Note',
    component: () => import('@/views/Note.vue'),
    meta: { title: '笔记', keepAlive: true }
  },
  {
    path: '/term',
    name: 'Terminal',
    component: () => import('@/views/Terminal.vue'),
    meta: { title: '终端', keepAlive: true }
  },
  {
    path: '/setting',
    name: 'setting',
    component: () => import('@/views/Setting.vue'),
    meta: { title: '设置', keepAlive: true }
  }
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes
})

export default router
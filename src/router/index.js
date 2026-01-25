import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    redirect: '/note'
  },
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue'),
    children: [
      {
        path: '/note',
        name: 'Note',
        component: () => import('@/views/Note.vue')
      },
      {
        path: '/term',
        name: 'Terminal',
        component: () => import('@/views/Terminal.vue')
      },
      {
        path: '/setting',
        name: 'Setting',
        component: () => import('@/views/Setting.vue')
      }
    ]
  }
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes
})

export default router
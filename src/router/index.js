import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue'),
    children: [
      {
        path: '/term',
        name: 'Terminal',
        component: () => import('@/views/Terminal.vue')
      },
      {
        path: '/note',
        name: 'Note',
        component: () => import('@/views/Note.vue')
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
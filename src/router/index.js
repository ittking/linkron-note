import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/login",
    name: "Login",
    component: () => import("@/Login.vue"),
    meta: { title: "登录" },
  },
  {
    path: "/",
    redirect: "/note",
    component: () => import("@/Main.vue"),
    meta: { requiresAuth: true },
    children: [
      {
        path: "/note",
        name: "Note",
        component: () => import("@/views/Note.vue"),
        meta: { title: "笔记", keepAlive: true, requiresAuth: true },
      },
      {
        path: "/todo",
        name: "Todo",
        component: () => import("@/views/Todo.vue"),
        meta: { title: "待办", keepAlive: true, requiresAuth: true },
      },
      {
        path: "/setting",
        name: "setting",
        component: () => import("@/views/Setting.vue"),
        meta: { title: "设置", keepAlive: true, requiresAuth: true },
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

// 检查登录状态
function checkAuth() {
  return !!localStorage.getItem('token')
}

// 路由守卫 - 检查登录状态
router.beforeEach((to, from, next) => {
  const isLoggedIn = checkAuth()

  // 如果访问的页面需要登录
  if (to.meta?.requiresAuth) {
    if (isLoggedIn) {
      next()
    } else {
      // 未登录，跳转到登录页
      next({ name: 'Login', query: { redirect: to.fullPath } })
    }
  } else if (to.name === 'Login' && isLoggedIn) {
    // 已登录用户访问登录页，跳转到主页
    next({ name: 'Note' })
  } else {
    next()
  }
})

export default router

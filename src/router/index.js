import { createRouter, createWebHistory } from "vue-router";

const routes = [
  {
    path: "/",
    redirect: "/note",
    component: () => import("@/Main.vue"),
    children: [
      {
        path: "/note",
        name: "Note",
        component: () => import("@/views/Note.vue"),
        meta: { title: "笔记", keepAlive: true },
      },
      {
        path: "/todo",
        name: "Todo",
        component: () => import("@/views/Todo.vue"),
        meta: { title: "待办", keepAlive: true },
      },
      {
        path: "/setting",
        name: "setting",
        component: () => import("@/views/Setting.vue"),
        meta: { title: "设置", keepAlive: true },
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

export default router;

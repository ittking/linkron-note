import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { useReminder } from "./composables/useReminder";

import "./style.css";
import "./assets/tiptap.css";

const app = createApp(App);

app.use(router);

app.mount("#app");

// 启动提醒检查任务
const { startReminderCheck } = useReminder()
startReminderCheck()
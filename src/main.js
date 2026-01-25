import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";

import "./style.css";

const app = createApp(App);

// 使用 Router
app.use(router);

app.mount("#app");

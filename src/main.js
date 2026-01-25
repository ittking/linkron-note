import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";

import "./style.css";
import Antd from "ant-design-vue";
import "ant-design-vue/dist/reset.css";

const app = createApp(App);

// 使用 Ant Design Vue
app.use(Antd);

// 使用 Router
app.use(router);

app.mount("#app");

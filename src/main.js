import { createApp } from "vue";
import App from "./App.vue";

// 导入 Tailwind CSS
import "./assets/main.css";

// 导入 Ant Design Vue
import Antd from "ant-design-vue";
import "ant-design-vue/dist/reset.css";

const app = createApp(App);

// 使用 Ant Design Vue
app.use(Antd);

app.mount("#app");

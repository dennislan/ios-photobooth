import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './styles/main.css';

// 在 Tauri 环境中，确保全局错误处理
const app = createApp(App);
app.use(createPinia());
app.mount('#app');

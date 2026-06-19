import { createApp } from 'vue';
import { createPinia } from 'pinia';
import Antd from 'ant-design-vue';
import App from './App.vue';
import './styles/tailwind.css';

// In Tauri environment, ensure global error handling
const app = createApp(App);
app.use(createPinia());
app.use(Antd);
app.mount('#app');

import { createApp } from "vue";
import App from "./App.vue";
import '@imengyu/vue3-context-menu/lib/vue3-context-menu.css'
import ContextMenu from '@imengyu/vue3-context-menu';

createApp(App).use(ContextMenu).mount('#app');

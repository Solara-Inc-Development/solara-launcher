import {createApp} from 'vue';
import App from './App.vue';
import '../../packages/ui/base.css';
import './launcher.css';

// Disable default Chromium/WebView2 browser context menu globally
window.addEventListener('contextmenu', (e) => e.preventDefault(), { capture: true });

createApp(App).mount('#app');


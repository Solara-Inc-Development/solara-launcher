import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'node:path';
export default defineConfig({plugins:[vue()],server:{port:1420,strictPort:true},build:{rollupOptions:{input:{website:resolve(__dirname,'index.html'),desktop:resolve(__dirname,'desktop.html')}}}});

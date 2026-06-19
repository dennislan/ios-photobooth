import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],

  css: {
    postcss: true,
  },

  // Tauri requires the dev server to listen on 0.0.0.0
  server: {
    host: "0.0.0.0",
    port: 1420,
  },

  build: {
    target: "esnext",
    outDir: "src-tauri/gen",
  },
});

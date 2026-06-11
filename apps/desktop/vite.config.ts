import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";

// Tauri espera porta fixa e não limpa a tela.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  resolve: {
    // Alias "@" → ./src (espelha o paths do tsconfig; Vite não lê tsconfig sozinho).
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  build: { target: "es2022", outDir: "dist" },
});

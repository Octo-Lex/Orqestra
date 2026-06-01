import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

const host = process.env.TAURI_DEV_HOST;
const browserTest = process.env.BROWSER_TEST === "1";

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  resolve: browserTest
    ? {
        alias: {
          "@tauri-apps/api/core": path.resolve(
            __dirname,
            "src/__mocks__/@tauri-apps_api_core.ts"
          ),
          "@tauri-apps/plugin-dialog": path.resolve(
            __dirname,
            "src/__mocks__/@tauri-apps_plugin-dialog.ts"
          ),
        },
      }
    : {},
  server: {
    port: browserTest ? 1421 : 1420,
    strictPort: !browserTest,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));

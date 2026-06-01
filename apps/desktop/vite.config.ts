import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import fs from "fs";

const host = process.env.TAURI_DEV_HOST;

// Check BROWSER_TEST from process env OR from .env.local file
let browserTest = process.env.BROWSER_TEST === "1";
if (!browserTest) {
  try {
    const envPath = path.resolve(__dirname, ".env.local");
    if (fs.existsSync(envPath)) {
      const envContent = fs.readFileSync(envPath, "utf-8");
      browserTest = envContent.includes("BROWSER_TEST=1");
    }
  } catch {
    // ignore
  }
}

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

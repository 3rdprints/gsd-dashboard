import path from "path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  base: "./",
  plugins: [react(), tailwindcss()],
  build: {
    outDir: "src-tauri/dist",
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            {
              name: "react-vendor",
              test: /node_modules[\\/](react|react-dom|scheduler)[\\/]/,
              priority: 40
            },
            {
              name: "charts-vendor",
              test: /node_modules[\\/](recharts|d3-[^\\/]+|victory-vendor|react-calendar-heatmap)[\\/]/,
              priority: 30
            },
            {
              name: "ui-vendor",
              test: /node_modules[\\/](radix-ui|@radix-ui|lucide-react|class-variance-authority|clsx|tailwind-merge)[\\/]/,
              priority: 20
            },
            {
              name: "app-vendor",
              test: /node_modules[\\/](@tanstack|@tauri-apps|react-router|react-router-dom|zustand)[\\/]/,
              priority: 10
            }
          ]
        }
      }
    }
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src")
    }
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: "./src/test/setup.ts"
  }
});

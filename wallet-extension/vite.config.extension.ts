import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";
import { copyFileSync, mkdirSync, existsSync } from "fs";

/**
 * Vite config for building the Sultan Wallet as a browser extension
 */
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    {
      name: 'copy-extension-files',
      closeBundle() {
        const distDir = path.resolve(process.cwd(), 'dist-extension');
        const extDir = path.resolve(process.cwd(), 'extension');
        const publicDir = path.resolve(process.cwd(), 'public');
        
        // Ensure dist exists
        if (!existsSync(distDir)) {
          mkdirSync(distDir, { recursive: true });
        }

        // Copy extension scripts
        const extensionFiles = [
          'background.js',
          'content-script.js', 
          'inpage-provider.js'
        ];
        
        for (const file of extensionFiles) {
          const src = path.join(extDir, file);
          const dest = path.join(distDir, file);
          if (existsSync(src)) {
            copyFileSync(src, dest);
            console.log(`Copied ${file} to dist-extension`);
          }
        }

        // Copy manifest
        const manifestSrc = path.join(publicDir, 'manifest.json');
        const manifestDest = path.join(distDir, 'manifest.json');
        if (existsSync(manifestSrc)) {
          copyFileSync(manifestSrc, manifestDest);
          console.log('Copied manifest.json to dist-extension');
        }
      }
    }
  ],
  resolve: {
    alias: {
      "@": path.resolve(process.cwd(), "src"),
      "@shared": path.resolve(process.cwd(), "shared"),
      "@assets": path.resolve(process.cwd(), "attached_assets"),
    },
  },
  root: process.cwd(),
  build: {
    outDir: "dist-extension",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        popup: path.resolve(process.cwd(), 'index.html'),
      },
      output: {
        entryFileNames: 'assets/[name].js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name].[ext]'
      }
    }
  },
  // Extension doesn't need dev server features
  define: {
    'process.env.IS_EXTENSION': JSON.stringify(true)
  }
});

import { defineConfig } from 'vite'
import { viteStaticCopy } from 'vite-plugin-static-copy'
import react from '@vitejs/plugin-react-swc'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
      react(),
      viteStaticCopy({
      targets: [
        {
          src: 'src/assets/logo.svg',
          dest: 'images'
        }
      ]
    })
  ],
})

import tailwindcss    from '@tailwindcss/vite';
import expressiveCode from 'astro-expressive-code';
import {defineConfig} from 'astro/config';

// eslint-disable-next-line arca/no-default-export
export default defineConfig({
  integrations: [
    expressiveCode(),
  ],

  vite: {
    plugins: [tailwindcss()],
  },
});

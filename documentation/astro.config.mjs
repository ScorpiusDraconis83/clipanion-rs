import {clipanionRemark} from '@clipanion/remark';
import tailwindcss       from '@tailwindcss/vite';
import expressiveCode    from 'astro-expressive-code';
import {defineConfig}    from 'astro/config';
import path              from 'node:path';

// eslint-disable-next-line arca/no-default-export
export default defineConfig({
  markdown: {
    remarkPlugins: [
      [clipanionRemark, {
        clis: {
          git: {
            baseUrl: `https://example.org/git`,
            path: path.resolve(import.meta.dirname, `../target/debug/clipanion-demo`),
          },
        },
        enableBlocks: false,
      }],
    ],
  },

  integrations: [
    expressiveCode(),
  ],

  vite: {
    plugins: [tailwindcss()],
  },
});

import { defineConfig } from 'tsup';

// eslint-disable-next-line arca/no-default-export
export default defineConfig([{
  entry: [`packages/clipanion-tools/index.ts`],
  outDir: `packages/clipanion-tools/dist`,
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
}, {
  entry: [`packages/clipanion-expressive-code/index.ts`, `packages/clipanion-expressive-code/extra.ts`],
  outDir: `packages/clipanion-expressive-code/dist`,
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
}, {
  entry: [`packages/clipanion-remark/index.ts`],
  outDir: `packages/clipanion-remark/dist`,
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
}, {
  entry: [`packages/clipanion-astro/index.ts`],
  outDir: `packages/clipanion-astro/dist`,
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  external: [`astro:content`],
}]);

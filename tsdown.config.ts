import fs from 'fs';
import path from 'path';
import { defineConfig, type Options } from 'tsdown';

const baseConfig = {
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
};

const packages = [
  `packages/clipanion-tools`,
  `packages/clipanion-expressive-code`,
  `packages/clipanion-remark`,
  `packages/clipanion-astro`,
];

const configs: Options[] = [];

for (const folder of packages) {
  const pkgDir = path.join(import.meta.dirname, folder);
  const pkgJson = JSON.parse(fs.readFileSync(path.join(pkgDir, 'package.json'), 'utf8'));

  const entry = Object.values(pkgJson.exports).flatMap((entry: any) => [
    entry,
    entry?.default,
    entry?.module,
  ]).filter(entry => {
    return typeof entry === 'string' && entry.match(/(?<!\.d)\.tsx?$/);
  }).map(entryPath => {
    return path.join(pkgDir, entryPath);
  });

  const external = [
    ...Object.keys(pkgJson.dependencies || {}),
    ...Object.keys(pkgJson.peerDependencies || {}),
  ];

  for (const format of [`esm`, `cjs`] as const) {
    configs.push({
      ...baseConfig,
      format,
      entry,
      outDir: path.join(pkgDir, 'dist'),
      external,
    });
  }
}

// eslint-disable-next-line arca/no-default-export
export default defineConfig(configs);

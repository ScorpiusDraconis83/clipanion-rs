import { defineCollection } from 'astro:content';
import { clipanionLoader } from '@clipanion/astro';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

const git = defineCollection({
  loader: clipanionLoader({
    name: `git`,
    path: require.resolve(`@clipanion/monorepo/target/debug/clipanion-demo`),
  }),
});

export const collections = {
  git,
};

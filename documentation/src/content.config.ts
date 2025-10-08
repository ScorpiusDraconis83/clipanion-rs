import {defineCliCollection} from '@clipanion/astro';
import {createRequire}       from 'module';

const require = createRequire(import.meta.url);

const git = defineCliCollection({
  path: require.resolve(`@clipanion/monorepo/target/debug/clipanion-demo`),
});

export const collections = {
  git,
};

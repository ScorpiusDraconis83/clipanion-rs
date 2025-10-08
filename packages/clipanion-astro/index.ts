import {ClipanionBinary}  from '@clipanion/tools';
import type {Loader}      from 'astro/loaders';
import {defineCollection} from 'astro:content';

export type CliOptions = {
  path: string;
};

export function defineCliCollection({path}: CliOptions) {
  const loader: Loader = {
    name: `@clipanion/astro`,
    load: async ({store}) => {
      const binary = new ClipanionBinary(path);
      const commandSpecs = await binary.commands();

      for (const commandSpec of commandSpecs) {
        store.set({
          id: commandSpec.primaryPath.join(`/`),
          data: commandSpec,
        });
      }
    },
  };

  return defineCollection({
    loader,
  });
}

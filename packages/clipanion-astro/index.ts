import { ClipanionBinary, CommandSpec } from '@clipanion/tools';
import { DataEntry as AstroDataEntry } from 'astro/content/config';
import type { Loader } from 'astro/loaders';

export type BaseData = {
  binaryName: string;
  commandSpec: CommandSpec;
};

export type DataEntry<T extends Record<string, unknown> = Record<string, unknown>> = {
  id: string;
  data: T;
  filePath?: string;
};

export type CliOptions<T extends Record<string, unknown>> = {
  id?: string;
  name: string;
  path: string;
  entry?: (entry: DataEntry<BaseData>) => DataEntry<T> | Promise<DataEntry<T>>;
  body?: (data: DataEntry<T>) => string;
};

export function clipanionLoaders<T extends Record<string, unknown> = BaseData>(opts: CliOptions<T>) {
  const binary = new ClipanionBinary(opts.path);
  const commandSpecs = binary.commands();

  return {
    commands: createCommandLoader<T>(opts, commandSpecs),
  };
};

function createCommandLoader<T extends Record<string, unknown> = BaseData>({ id, name, entry: entryFn, body: bodyFn }: CliOptions<T>, commandSpecs: Promise<CommandSpec[]>): Loader {
  return {
    name: `@clipanion/astro`,

    load: async ({ renderMarkdown, store }) => {
      store.clear();
      for (const commandSpec of await commandSpecs) {
        const baseData: BaseData = {
          binaryName: name,
          commandSpec,
        };

        const baseEntry = {
          id: [id ?? name, ...commandSpec.primaryPath].join(`/`),
          data: baseData,
        };

        const entry = entryFn
          ? await entryFn(baseEntry)
          : baseEntry as any as DataEntry<T>;

        const body = bodyFn?.(entry);

        const rendered = body
          ? await renderMarkdown(body)
          : undefined;

        store.set({
          ...entry,
          body,
          rendered,
        });
      }
    },
  };
}

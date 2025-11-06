import {ClipanionBinary, type CommandSpec} from '@clipanion/tools';
import type {Loader}                       from 'astro/loaders';

export {type CommandSpec} from '@clipanion/tools';

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
  specCommand?: Array<string>;
  filter?: (entry: DataEntry<BaseData>) => boolean;
  entry?: (entry: DataEntry<BaseData>) => DataEntry<T> | Promise<DataEntry<T>>;
  body?: (data: DataEntry<T>) => string;
};

export function clipanionLoaders<T extends Record<string, unknown> = BaseData>(opts: CliOptions<T>) {
  const binary = new ClipanionBinary(opts.path, {
    specCommand: opts.specCommand,
  });

  return {
    commands: createCommandLoader<T>(opts, binary),
  };
}

function createCommandLoader<T extends Record<string, unknown> = BaseData>({id, name, filter: filterFn, entry: entryFn, body: bodyFn}: CliOptions<T>, binary: ClipanionBinary): Loader {
  return {
    name: `@clipanion/astro`,

    load: async ({renderMarkdown, store}) => {
      const sync = async () => {
        store.clear();

        for (const commandSpec of await binary.commands()) {
          const baseData: BaseData = {
            binaryName: name,
            commandSpec,
          };

          const baseEntry = {
            id: [id ?? name, ...commandSpec.primaryPath].join(`/`),
            data: baseData,
          };

          if (filterFn?.(baseEntry) === false)
            continue;

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
      };

      binary.onUpdate(async () => {
        await sync();
      });

      await sync();
    },
  };
}

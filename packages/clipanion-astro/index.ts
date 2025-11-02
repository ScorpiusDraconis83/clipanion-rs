import { ClipanionBinary, CommandSpec } from '@clipanion/tools';
import type { Loader } from 'astro/loaders';

export type BaseData = {
  binaryName: string;
  commandSpec: CommandSpec;
};

export type Data<TExtraData extends Record<string, unknown>> = BaseData & TExtraData;

export type CliOptions<TExtraData extends Record<string, unknown>> = {
  id?: string;
  name: string;
  path: string;
  extraData?: (data: BaseData) => TExtraData | Promise<TExtraData>;
  template?: (data: Data<TExtraData>) => string;
};

export function clipanionLoader<TExtraData extends Record<string, unknown> = {}>({ id, name, path, extraData: extraDataFn, template: templateFn }: CliOptions<TExtraData>): Loader {
  return {
    name: `@clipanion/astro`,

    load: async ({ renderMarkdown, store }) => {
      const binary = new ClipanionBinary(path);
      const commandSpecs = await binary.commands();

      for (const commandSpec of commandSpecs) {
        const baseData: BaseData = {
          binaryName: name,
          commandSpec,
        };

        const extraData = extraDataFn
          ? await extraDataFn(baseData)
          : {} as TExtraData;

        const data: Data<TExtraData> = { ...baseData, ...extraData };

        const rendered = templateFn
          ? await renderMarkdown(templateFn(data))
          : undefined;

        store.set({
          id: [id ?? name, ...commandSpec.primaryPath].join(`/`),
          data,
          rendered,
        });
      }
    },
  };
}

import { ClipanionBinary } from '@clipanion/tools';
import type { Loader } from 'astro/loaders';
import { z } from 'astro/zod';

const exampleType = z.object({
  command: z.string(),
  description: z.string().nullable(),
});

const keywordPositionalType = z.object({
  positionalType: z.literal(`keyword`),
  expected: z.string(),
});

const dynamicPositionalType = z.object({
  positionalType: z.literal(`dynamic`),

  name: z.string(),
  description: z.string().nullable(),

  minLen: z.number(),
  extraLen: z.number().nullable(),

  isPrefix: z.boolean(),
  isProxy: z.boolean(),
});

const optionType = z.object({
  type: z.literal(`option`),

  primaryName: z.string(),
  aliases: z.array(z.string()),

  description: z.string().nullable(),

  minLen: z.number(),
  extraLen: z.number().nullable(),

  allowBinding: z.boolean(),
  allowBoolean: z.boolean(),
  isHidden: z.boolean(),
  isRequired: z.boolean(),
});

const componentType = z.union([
  z.intersection(z.object({
    type: z.literal(`positional`),
  }), z.union([
    keywordPositionalType,
    dynamicPositionalType,
  ])),
  optionType,
]);

const commandSpecType = z.object({
  primaryPath: z.array(z.string()),
  aliases: z.array(z.array(z.string())),
  category: z.string().nullable(),
  description: z.string().nullable(),
  details: z.string().nullable(),
  examples: z.array(exampleType),
  components: z.array(componentType),
  requiredOptions: z.array(z.number()),
});

const loaderSchema = z.object({
  binaryName: z.string(),
  commandSpec: commandSpecType,
});

export type CliOptions = {
  name: string;
  path: string;
};

export function clipanionLoader({ name, path }: CliOptions): Loader {
  return {
    name: `@clipanion/astro`,
    schema: loaderSchema,
    load: async ({ store }) => {
      const binary = new ClipanionBinary(path);
      const commandSpecs = await binary.commands();

      for (const commandSpec of commandSpecs) {
        const data: z.infer<typeof loaderSchema> = {
          binaryName: name,
          commandSpec,
        };

        store.set({
          id: commandSpec.primaryPath.join(`/`),
          data,
        });
      }
    },
  };
}

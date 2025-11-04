import { clipanionLoaders } from '@clipanion/astro';
import { defineCollection } from 'astro:content';
import dedent from 'dedent';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);

const gitLoaders = clipanionLoaders({
  name: `git`,
  path: require.resolve(`@clipanion/monorepo/target/debug/clipanion-demo`),

  entry: entry => ({
    ...entry,
    data: {
      ...entry.data,
      sidebar: {
        label: `${entry.data.binaryName} ${entry.data.commandSpec.primaryPath.join(` `)}`,
      },
    },
  }),

  body: ({ data: { binaryName, commandSpec, sidebar } }) => {
    const options = commandSpec.components
      .filter((component): component is Extract<typeof component, { type: `option` }> => component.type === `option` && !component.isHidden);

    return dedent.withOptions({ alignValues: true })`
      ## ${sidebar.label}

      ${commandSpec.documentation?.description}

      \`\`\`bash
      ${binaryName} ${commandSpec.primaryPath.join(` `)}
      \`\`\`

      ${commandSpec.documentation?.details}

      ${options.length > 0 ? dedent`
        ### Options

        ${options.map(option => dedent`
          #### ${option.primaryName}
          ${option.documentation?.description}
        `).join(`\n`)}
      ` : ``}
    `;
  },
});

const docs = defineCollection({
  loader: gitLoaders.commands,
});

export const collections = {
  docs,
};

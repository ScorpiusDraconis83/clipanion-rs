import { defineCollection } from 'astro:content';
import { clipanionLoader } from '@clipanion/astro';
import { createRequire } from 'module';
import dedent from 'dedent';

const require = createRequire(import.meta.url);

const docs = defineCollection({
  loader: clipanionLoader({
    name: `git`,
    path: require.resolve(`@clipanion/monorepo/target/debug/clipanion-demo`),

    extraData: ({ binaryName, commandSpec }) => ({
      sidebar: {
        label: `${binaryName} ${commandSpec.primaryPath.join(` `)}`,
      },
    }),

    template: ({ binaryName, commandSpec, sidebar }) => {
      const options = commandSpec.components
        .filter((component): component is Extract<typeof component, { type: 'option' }> => component.type === `option` && !component.isHidden);

      return dedent`
        ## ${sidebar.label}

        ${commandSpec.description}

        \`\`\`bash
        ${binaryName} ${commandSpec.primaryPath.join(` `)}
        \`\`\`

        ${options.length > 0 && dedent`
          ### Options

          ${options.map(option => dedent`
            #### ${option.primaryName}
            ${option.description}
          `).join(`\n`)}
        `}
      `;
    },
  }),
});

export const collections = {
  docs,
};

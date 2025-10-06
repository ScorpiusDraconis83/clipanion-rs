import {optimizer, tooltip}      from '@clipanion/expressive-code/extra';
import {clipanionExpressiveCode} from '@clipanion/expressive-code';
import expressiveCode            from 'astro-expressive-code';
import {defineConfig}            from 'astro/config';

// eslint-disable-next-line arca/no-default-export
export default defineConfig({
  integrations: [
    expressiveCode({
      plugins: [
        tooltip(),
        optimizer({
          mergeOrder: [`data-tooltip`, `style`],
        }),
        clipanionExpressiveCode({
          clis: {
            [`clipanion-demo`]: {
              baseUrl: `https://example.org/ssh`,
              path: `${import.meta.dirname}/../target/debug/clipanion-demo`,
            },
          },
        }),
      ],
    }),
  ],
});

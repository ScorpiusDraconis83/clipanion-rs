# @clipanion/expressive-code

A plugin for [Expressive Code](https://github.com/expressive-code/expressive-code) to add semantic annotations for Clipanion CLI commands.

## Usage

Make sure your CLI is available on disk, then register it inside the plugin's configuration (here taking the Astro configuration file as example):

```ts
import {optimizer, tooltip}      from '@clipanion/expressive-code/extra';
import {clipanionExpressiveCode} from '@clipanion/expressive-code';

export default defineConfig({
  integrations: [
    expressiveCode({
      plugins: [
        clipanionExpressiveCode({
          clis: {
            [`my-cli`]: {
              baseUrl: `https://example.org/my-cli`,
              path: `/path/to/my-cli`,
            },
          },
        }),

        // Optional; merges attributes with similar `style` values
        // Optimizes the html output and fixes some visual underline glitches
        optimizer(),

        // Required for tooltips to work
        tooltip(),
      ],
    }),
  ],
});
```

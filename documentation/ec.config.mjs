import {optimizer, tooltip}      from '@clipanion/expressive-code/extra';
import {clipanionExpressiveCode} from '@clipanion/expressive-code';
import {createRequire}           from 'module';

const require = createRequire(import.meta.url);

// eslint-disable-next-line arca/no-default-export
export default {
  plugins: [
    tooltip(),
    optimizer(),
    clipanionExpressiveCode({
      clis: {
        [`git`]: {
          baseUrl: `https://example.org/git`,
          path: require.resolve(`@clipanion/monorepo/target/debug/clipanion-demo`),
        },
      },
    }),
  ],
};

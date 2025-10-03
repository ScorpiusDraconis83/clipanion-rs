import yarnConfig from '@yarnpkg/eslint-config';

// eslint-disable-next-line arca/no-default-export
export default [
  {
    ignores: [
      `.yarn`,
      `packages/clipanion-tools/types`,
    ],
  },
  ...yarnConfig,
];

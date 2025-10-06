import {ClipanionBinary} from '.';

describe(`ClipanionBinary`, () => {
  it(`should describe the command line`, async () => {
    const binary = new ClipanionBinary(`${__dirname}/../../target/debug/clipanion-demo`);
    const result = await binary.describeCommandLine([`ssh`, `-p`, `80`, `localhost`]);

    expect(result).toEqual({
      command: [`ssh`],
      annotations: [
        {type: `keyword`, start: {tokenIndex: 0, argIndex: 0, offset: 0}, end: {tokenIndex: 0, argIndex: 0, offset: 3}, description: `Connect to a host`},
        {type: `option`, start: {tokenIndex: 1, argIndex: 1, offset: 0}, end: {tokenIndex: 2, argIndex: 2, offset: 2}, description: `Port to connect to`},
        {type: `positional`, start: {tokenIndex: 3, argIndex: 3, offset: 0}, end: {tokenIndex: 3, argIndex: 3, offset: 9}, description: `Host to connect to`},
      ],
      tokens: [
        {type: `keyword`, argIndex: 0, slice: {start: 0, end: 3}},
        {type: `option`, argIndex: 1, slice: {start: 0, end: 2}, componentId: 0},
        {type: `value`, argIndex: 2, slice: {start: 0, end: 2}, componentId: 0},
        {type: `positional`, argIndex: 3, slice: {start: 0, end: 9}, componentId: 4},
      ],
    });
  });
});

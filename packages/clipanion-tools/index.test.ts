import { ClipanionBinary } from ".";

describe(`ClipanionBinary`, () => {
  it(`should describe the command line`, async () => {
    const binary = new ClipanionBinary(`${__dirname}/../../target/debug/clipanion-demo`);
    const result = await binary.describeCommandLine([`ssh`, `-p`, `80`, `localhost`]);

    expect(result).toEqual([
      {text: `ssh`, tags: {type: `keyword`, description: `Connect to a host`}},
      {text: ` `, tags: {type: `unknown`, description: null}},
      {text: `-p`, tags: {type: `option`, description: `Port to connect to`}},
      {text: ` `, tags: {type: `unknown`, description: null}},
      {text: `80`, tags: {type: `value`, description: `Port to connect to`}},
      {text: ` `, tags: {type: `unknown`, description: null}},
      {text: `localhost`, tags: {type: `positional`, description: `Host to connect to`}},
    ]);
  });
});

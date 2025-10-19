import remarkParse       from 'remark-parse';
import remarkStringify   from 'remark-stringify';
import {unified}         from 'unified';

import {clipanionRemark} from './index.js';

describe(`clipanionRemark`, () => {
  const demoBinaryPath = `${__dirname}/../../target/debug/clipanion-demo`;
  const cliConfig = {
    git: {
      path: demoBinaryPath,
    },
  };

  describe(`plugin options`, () => {
    it(`should process code blocks when enableBlocks is true (default)`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit commit -m msg\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should process the command and potentially add annotations
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">git <span style="color: var(--cli-color-block-keyword);" data-tooltip="Record changes to the repository.">commit</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Use the given message as the commit message.">-m msg</span></div>
        "
      `);
    });

    it(`should skip code blocks when enableBlocks is false`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
          enableBlocks: false,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit commit -m msg\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should not process the command
      expect(output).toMatchInlineSnapshot(`
        "\`\`\`bash
        git commit -m msg
        \`\`\`
        "
      `);
    });

    it(`should process inline code when enableInline is true (default)`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`git commit -m msg\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should process the command
      expect(output).toMatchInlineSnapshot(`
        "<code>git <span style="color: var(--cli-color-block-keyword);" data-tooltip="Record changes to the repository.">commit</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Use the given message as the commit message.">-m msg</span></code>
        "
      `);
    });

    it(`should skip inline code when enableInline is false`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
          enableInline: false,
        })
        .use(remarkStringify);

      const input = `\`git commit -m msg\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should not process the command
      expect(output).toMatchInlineSnapshot(`
        "\`git commit -m msg\`
        "
      `);
    });
  });

  describe(`code block processing`, () => {
    it(`should process bash code blocks`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit commit -m msg\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">git <span style="color: var(--cli-color-block-keyword);" data-tooltip="Record changes to the repository.">commit</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Use the given message as the commit message.">-m msg</span></div>
        "
      `);
    });

    it(`should skip non-bash code blocks`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`javascript\nconsole.log("hello");\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should not process JavaScript code
      expect(output).toMatchInlineSnapshot(`
        "\`\`\`javascript
        console.log("hello");
        \`\`\`
        "
      `);
    });

    it(`should skip code blocks without language`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`\nsome code\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should not process code without language
      expect(output).toMatchInlineSnapshot(`
        "\`\`\`
        some code
        \`\`\`
        "
      `);
    });
  });

  describe(`command line parsing`, () => {
    it(`should parse git add command with options`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit add --verbose --dry-run file.txt\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should process the command and potentially highlight options
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">git add --verbose --dry-run file.txt</div>
        "
      `);
    });

    it(`should parse git commit command`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit commit --amend -m "fix: update tests"\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should process the command
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">git <span style="color: var(--cli-color-block-keyword);" data-tooltip="Record changes to the repository.">commit</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Replace the tip of the current branch by creating a new commit.">--amend</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Use the given message as the commit message.">-m "fix: update tests"</span></div>
        "
      `);
    });

    it(`should parse git rm command`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit rm --force --cached file.txt\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should process the command
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">git <span style="color: var(--cli-color-block-keyword);" data-tooltip="Remove files from the working tree and from the index.">rm</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Override the up-to-date check.">--force</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Unstage and remove paths only from the index.">--cached</span> <span style="color: var(--cli-color-block-positional);" data-tooltip="Files to remove.">file.txt</span></div>
        "
      `);
    });

    it(`should handle invalid commands gracefully`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ninvalid-command --option\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should not crash and should preserve the original text
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">invalid-command --option</div>
        "
      `);
    });
  });

  describe(`HTML generation`, () => {
    it(`should generate HTML for code blocks`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit commit -m msg\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should generate HTML structure
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">git <span style="color: var(--cli-color-block-keyword);" data-tooltip="Record changes to the repository.">commit</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Use the given message as the commit message.">-m msg</span></div>
        "
      `);
    });

    it(`should generate HTML for inline code`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`git commit -m msg\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should generate HTML structure
      expect(output).toMatchInlineSnapshot(`
        "<code>git <span style="color: var(--cli-color-block-keyword);" data-tooltip="Record changes to the repository.">commit</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Use the given message as the commit message.">-m msg</span></code>
        "
      `);
    });
  });

  describe(`edge cases`, () => {
    it(`should handle empty code blocks`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should handle empty blocks gracefully
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block"></div>
        "
      `);
    });

    it(`should handle whitespace-only code blocks`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\n   \n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should handle whitespace gracefully
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block"></div>
        "
      `);
    });

    it(`should handle multiple lines in code blocks`, async () => {
      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliConfig,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit add file1.txt\ngit commit -m "message"\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should process multiple lines
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">git add file1.txt
        git <span style="color: var(--cli-color-block-keyword);" data-tooltip="Record changes to the repository.">commit</span> <span style="color: var(--cli-color-block-option);" data-tooltip="Use the given message as the commit message.">-m "message"</span></div>
        "
      `);
    });

    it(`should handle CLI configuration with baseUrl`, async () => {
      const cliWithBaseUrl = {
        git: {
          path: demoBinaryPath,
          baseUrl: `https://example.com`,
        },
      };

      const processor = unified()
        .use(remarkParse)
        .use(clipanionRemark, {
          clis: cliWithBaseUrl,
        })
        .use(remarkStringify);

      const input = `\`\`\`bash\ngit commit -m msg\n\`\`\``;
      const result = await processor.process(input);
      const output = result.toString();

      // Should process with baseUrl configuration
      expect(output).toMatchInlineSnapshot(`
        "<div class="custom-code-block">git <a style="color: var(--cli-color-block-keyword);" data-tooltip="Record changes to the repository." href="https://example.com//commit" target="_blank">commit</a> <span style="color: var(--cli-color-block-option);" data-tooltip="Use the given message as the commit message.">-m msg</span></div>
        "
      `);
    });
  });
});

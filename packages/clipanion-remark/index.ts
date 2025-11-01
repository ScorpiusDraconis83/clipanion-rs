import { ClipanionBinary, parseCli } from '@clipanion/tools';
import type { Code, Html, InlineCode, Node } from 'mdast';
import { Transformer } from 'unified';
import { visit } from 'unist-util-visit';

export type PluginOptions = {
  clis: Record<string, {
    baseUrl?: string | undefined;
    path: string;
  }>;
  enableBlocks?: boolean;
  enableInline?: boolean;
};

type AnnotatedSlice = {
  start: number;
  end: number;
  prefix: string;
  suffix: string;
};

function applyAnnotation(source: string, annotations: Array<AnnotatedSlice>) {
  const groups = new Map<number, Array<AnnotatedSlice>>();

  for (const annotation of annotations) {
    const group = groups.get(annotation.start) ?? [];
    group.push(annotation);
    groups.set(annotation.start, group);
  }

  const sortedGroups = Array.from(groups.entries())
    .sort(([a], [b]) => b - a)
    .map(([_, group]) => group);

  for (const group of sortedGroups) {
    const { start, end } = group[0]!;

    let prefix = ``;
    let suffix = ``;

    for (const annotation of group) {
      prefix = `${prefix}${annotation.prefix}`;
      suffix = `${annotation.suffix}${suffix}`;
    }

    const left = source.slice(0, start);
    const middle = source.slice(start, end);
    const right = source.slice(end);

    source = left + prefix + middle + suffix + right;
  }

  return source;
}

export function clipanionRemark({ clis, enableBlocks = true, enableInline = true }: PluginOptions): Transformer {
  const binaries = Object.fromEntries(
    Object.entries(clis).map(([name, cli]) => [name, {
      ...cli,
      binary: new ClipanionBinary(cli.path),
    }]),
  );

  async function handleCodeBlock(node: Code | InlineCode): Promise<void> {
    if (node.type === `code` && node.lang !== `bash`)
      return;

    const lines = node.value
      .trim()
      .split(`\n`);

    const outputLines: Array<string> = [];

    for (const line of lines) {
      const result = await parseCli(line, binaries);
      if (!result) {
        outputLines.push(line);
        continue;
      }

      const {
        cli,
        words,
        query: {
          command,
          annotations,
        },
      } = result;

      const slices: Array<AnnotatedSlice> = [];

      for (const annotation of annotations) {
        const start = words[annotation.start.argIndex]!.index + annotation.start.offset;
        const end = words[annotation.end.argIndex]!.index + annotation.end.offset;

        let href: string | null = null;

        if (annotation.type === `keyword` && cli.baseUrl) {
          const targetUrl = new URL(cli.baseUrl);
          targetUrl.pathname += command.map(segment => `/${segment}`).join(``);
          href = targetUrl.toString();
        }

        const tagName = href !== null
          ? `a`
          : `span`;

        let attributes = `style="color: var(--cli-color-block-${annotation.type});"`;

        if (annotation.description !== null)
          attributes += ` data-tooltip="${annotation.description}"`;

        if (href !== null)
          attributes += ` href="${href}" target="_blank"`;

        slices.push({
          start,
          end,
          prefix: `<${tagName} ${attributes}>`,
          suffix: `</${tagName}>`,
        });
      }

      outputLines.push(applyAnnotation(line, slices));
    }

    const htmlContent = outputLines
      .join(`\n`);

    const value = node.type === `inlineCode`
      ? `<code>${htmlContent}</code>`
      : `<div class="custom-code-block">${htmlContent}</div>`;

    const nodeAsHtml = node as Node as Html;

    nodeAsHtml.type = `html`;
    nodeAsHtml.value = value;
  }

  return async tree => {
    const promises: Array<Promise<void>> = [];

    if (enableBlocks) {
      visit(tree, `code`, node => {
        promises.push(handleCodeBlock(node));
      });
    }

    if (enableInline) {
      visit(tree, `inlineCode`, node => {
        promises.push(handleCodeBlock(node));
      });
    }

    await Promise.all(promises);
  };
}

import {ClipanionBinary}                                                                                                                                                  from '@clipanion/tools';
import {type Element, type Parents, h}                                                                                                                                    from '@expressive-code/core/hast';
import {type AnnotationBaseOptions, type AnnotationRenderOptions, type AnnotationRenderPhase, type ExpressiveCodePlugin, ExpressiveCodeAnnotation, InlineStyleAnnotation} from '@expressive-code/core';

function wrapInto(children: Array<Parents>, wrapper: Element) {
  for (const child of children) {
    if (child.type === `root`) {
      for (const transitiveChild of child.children) {
        if (transitiveChild.type !== `doctype`) {
          wrapper.children.push(transitiveChild);
        }
      }
    } else {
      wrapper.children.push(child);
    }
  }

  const nextChildren: Array<Parents> = [wrapper];

  while (nextChildren.length < children.length)
    nextChildren.push(h(`span`));

  return nextChildren;
}

type ClipanionAnnotationOptions = AnnotationBaseOptions & {
  description: string | null;
  href: string | null;
};

class ClipanionAnnotation extends ExpressiveCodeAnnotation {
  readonly description: string | null;
  readonly href: string | null;

  override renderPhase: AnnotationRenderPhase = `latest`;

  constructor(options: ClipanionAnnotationOptions) {
    super(options);

    this.description = options.description;
    this.href = options.href;
  }

  override render({nodesToTransform}: AnnotationRenderOptions) {
    if (!this.description && !this.href)
      return nodesToTransform;

    const tagName = this.href !== null
      ? `a`
      : `span`;

    return wrapInto(nodesToTransform, h(tagName, {
      [`data-tooltip`]: this.description,
      [`href`]: this.href,
    }));
  }
}

export type PluginOptions = {
  clis: Record<string, {
    baseUrl?: string | undefined;
    path: string;
  }>;
};

export function clipanionExpressiveCode({clis}: PluginOptions): ExpressiveCodePlugin {
  const binaries = Object.fromEntries(
    Object.entries(clis).map(([name, cli]) => [name, {
      ...cli,
      binary: new ClipanionBinary(cli.path),
    }]),
  );

  return {
    name: `@clipanion/expressive-code`,
    hooks: {
      postprocessAnalyzedCode: async ({codeBlock}) => {
        if (codeBlock.language !== `bash`)
          return;

        for (const line of codeBlock.getLines()) {
          if (line.text.startsWith(`#`) || line.text.length === 0)
            continue;

          const words = [...line.text.matchAll(/"[^"]+"|'[^']+'|[^\s]+/g)];
          if (words.length === 0)
            continue;

          const cliName = words.shift()![0];
          if (!Object.hasOwn(binaries, cliName))
            continue;

          const cli = binaries[cliName]!;
          const args = words.map(word => word[0]);

          const query = await cli.binary.describeCommandLine(args);
          if (!query)
            continue;

          const {
            command,
            tokens,
            annotations,
          } = query;

          for (const annotation of annotations) {
            const inlineRange = {
              columnStart: words[annotation.start.argIndex]!.index + annotation.start.offset,
              columnEnd: words[annotation.end.argIndex]!.index + annotation.end.offset,
            };

            let href: string | null = null;

            if (annotation.type === `keyword` && cli.baseUrl) {
              const targetUrl = new URL(cli.baseUrl);
              targetUrl.pathname += command.map(segment => `/${segment}`).join(``);
              href = targetUrl.toString();
            }

            line.addAnnotation(
              new InlineStyleAnnotation({
                inlineRange,
                color: `var(--cli-color-block-${annotation.type})`,
              }),
            );

            line.addAnnotation(
              new ClipanionAnnotation({
                inlineRange,
                description: annotation.description,
                href,
              }),
            );
          }

          for (const token of tokens) {
            const inlineRange = {
              columnStart: words[token.argIndex]!.index + token.slice.start,
              columnEnd: words[token.argIndex]!.index + token.slice.end,
            };

            line.addAnnotation(
              new InlineStyleAnnotation({
                inlineRange,
                color: `var(--cli-color-block-${token.type})`,
              }),
            );
          }
        }
      },
    },
  };
}

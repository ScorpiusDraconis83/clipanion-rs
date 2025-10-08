import {type Element, type ElementContent, h} from '@expressive-code/core/hast';
import {type ExpressiveCodePlugin}            from '@expressive-code/core';

export type OptimizerOptions = {
  mergeOrder?: Array<string>;
};

export function optimizer({mergeOrder = [`className`, `style`]}: {mergeOrder?: Array<string>} = {}): ExpressiveCodePlugin {
  function processNode(node: ElementContent, attributes: Array<string>) {
    if (node.type === `element`) {
      for (const attribute of attributes)
        node.children = mergeChildren(node.children, attribute);

      for (const child of node.children) {
        processNode(child, attributes);
      }
    }
  }

  function mergeChildren(inChildren: Array<ElementContent>, attribute: string) {
    const outChildren: Array<ElementContent> = [];

    for (let t = 0; t < inChildren.length; t++) {
      const child = inChildren[t]!;

      if (child.type !== `element` || child.tagName !== `span` || !Object.hasOwn(child.properties, attribute)) {
        outChildren.push(child);
        continue;
      }

      const attributeValue = child.properties[attribute]!;
      const mergedElements: Array<ElementContent> = [];

      const pushElement = (element: Element) => {
        delete element.properties[attribute];

        if (Object.keys(element.properties).length > 0) {
          mergedElements.push(element);
        } else {
          mergedElements.push(...element.children);
        }
      };

      pushElement(child);

      while (t + 1 < inChildren.length) {
        const nextChild = inChildren[t + 1]!;
        if (nextChild.type !== `element`)
          break;

        if (nextChild.tagName !== `span`)
          break;

        const nextChildAttribute = nextChild.properties[attribute]!;
        if (nextChildAttribute !== attributeValue)
          break;

        pushElement(nextChild);
        t += 1;
      }

      outChildren.push(h(`span`, {[attribute]: attributeValue}, mergedElements));
    }

    return outChildren;
  }

  return {
    name: `@clipanion/expressive-code/optimizer`,
    hooks: {
      postprocessRenderedLine: ({renderData}) => {
        processNode(renderData.lineAst, mergeOrder);
      },
    },
  };
}

export function tooltip(): ExpressiveCodePlugin {
  return {
    name: `@clipanion/expressive-code/tooltip`,
    jsModules: [`
      for (const expressiveCodeEl of document.querySelectorAll('.expressive-code')) {
        const tooltipContainer = expressiveCodeEl.querySelector('.tooltip-container');
        const tooltip = tooltipContainer.querySelector('.tooltip');

        for (const annotatedEl of expressiveCodeEl.querySelectorAll('[data-tooltip]')) {
          annotatedEl.addEventListener('mouseenter', () => {
            const expressiveCodeRect = expressiveCodeEl.getBoundingClientRect();
            const annotatedRect = annotatedEl.getBoundingClientRect();

            tooltipContainer.style.left = annotatedRect.left - expressiveCodeRect.left + 'px';
            tooltipContainer.style.top = annotatedRect.top - expressiveCodeRect.top + 'px';
            tooltipContainer.style.width = annotatedRect.width + 'px';
            tooltipContainer.style.height = annotatedRect.height + 'px';
            tooltipContainer.style.display = 'block';

            tooltip.textContent = annotatedEl.getAttribute('data-tooltip');
          });

          annotatedEl.addEventListener('mouseleave', () => {
            tooltipContainer.style.display = 'none';
          });
        }
      }
    `],
    hooks: {
      postprocessRenderedBlock: async ({renderData}) => {
        renderData.blockAst.children.push(h(`div`, {
          className: `tooltip-container`,
        }, h(`div`, {
          className: `tooltip`,
        })));
      },
    },
  };
}

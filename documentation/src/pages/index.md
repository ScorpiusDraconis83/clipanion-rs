<style>
  :root {
    --cli-color-block-binary: #9e95fa;
    --cli-color-block-keyword: #9bd1f4;
    --cli-color-block-positional: #89b6db;
    --cli-color-block-syntax: #b0b2fe;
    --cli-color-block-option: #b0b2fe;
    --cli-color-block-assign: #b0b2fe;
    --cli-color-block-value: #d7a856;
  }

  .expressive-code .tooltip-container {
    display: none;
    position: absolute;
    pointer-events: none;
    z-index: 1000;
  }

  .expressive-code [data-tooltip] {
    text-decoration: underline !important;
    text-decoration-color: #000000;
    text-underline-offset: 2px;
  }

  .expressive-code [data-tooltip]:not(a) {
    text-decoration-style: dotted !important;
    cursor: help;
  }

  .expressive-code a {
    color: inherit;
  }

  .expressive-code .tooltip {
    position: absolute;
    left: 50%;
    bottom: 100%;
    margin-bottom: 8px;
    width: max-content;
    border-radius: 2px;
    padding: 4px 8px;
    background: rgba(0, 0, 0, 0.9);
    color: #ffffff;
    transform: translateX(-50%);
}

  .expressive-code .tooltip::after {
    content: '';

    position: absolute;
    top: 100%;
    left: 50%;

    margin-left: -5px;

    border-width: 5px;
    border-style: solid;
    border-color: rgba(0, 0, 0, 0.9) transparent transparent transparent;
  }
</style>

```bash
clipanion-demo sxsh --user=root localhost
```

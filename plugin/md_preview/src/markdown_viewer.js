import katex from "katex";
import {unified} from "unified";
import remarkGfm from "remark-gfm";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import rehypeStringify from "rehype-stringify";

const INLINE_OPEN = "\\(";
const INLINE_CLOSE = "\\)";
const DISPLAY_OPEN = "\\[";
const DISPLAY_CLOSE = "\\]";

export async function renderMarkdownToElement({text, element, basePath = "", macros = {}}) {
  const html = await renderMarkdownToHtml({text, basePath, macros});
  element.innerHTML = html;
  element.classList.add("md-view");
  await mountEmbeddedModels(element);
  return element;
}

export async function renderMarkdownToHtml({text, basePath = "", macros = {}}) {
  const katexMacros = normalizeKatexMacros(macros);
  const processor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkComputationExtensions)
    .use(remarkRehype, {
      allowDangerousHtml: true,
      handlers: {
        definitionList(state, node) {
          return {
            type: "element",
            tagName: "dl",
            properties: {
              className: ["md-definition-list"],
            },
            children: node.children.flatMap(item => definitionItemToHast(state, item)),
          };
        },
        inlineMath(state, node) {
          return {
            type: "raw",
            value: renderKatexNode(node, katexMacros, false),
          };
        },
        displayMath(state, node) {
          return {
            type: "raw",
            value: renderKatexNode(node, katexMacros, true),
          };
        },
        mathError(state, node) {
          return {
            type: "raw",
            value: renderMathError(node.value, node.message, false),
          };
        },
        alert(state, node) {
          const type = normalizeAlertType(node.alertType);
          return {
            type: "element",
            tagName: "div",
            properties: {
              className: ["md-alert", `md-alert-${type}`],
            },
            children: [
              {
                type: "element",
                tagName: "p",
                properties: {
                  className: ["md-alert-title"],
                },
                children: [{type: "text", value: alertLabel(type)}],
              },
              {
                type: "element",
                tagName: "div",
                properties: {
                  className: ["md-alert-body"],
                },
                children: state.all(node),
              },
            ],
          };
        },
        link(state, node) {
          const href = resolveMarkdownHref(node.url, basePath);
          return {
            type: "element",
            tagName: "a",
            properties: {href},
            children: state.all(node),
          };
        },
        image(state, node) {
          return {
            type: "element",
            tagName: "img",
            properties: {
              src: resolveAssetHref(node.url, basePath),
              alt: node.alt || "",
              title: node.title || undefined,
            },
            children: [],
          };
        },
      },
    })
    .use(rehypeStringify, {allowDangerousHtml: true});

  const file = await processor.process(escapeMathDelimiters(text));
  return String(file);
}

export function from_text(text) {
  return parseKatexMacros(text);
}

function parseKatexMacros(text) {
  const macros = {};
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#") || line.startsWith("%")) {
      continue;
    }

    const separator = line.indexOf(":");
    if (separator === -1) {
      continue;
    }

    const key = line.slice(0, separator).trim();
    const value = line.slice(separator + 1).trim();
    if (!key || !value) {
      continue;
    }
    macros[key] = value;
  }

  return macros;
}

function normalizeKatexMacros(macros) {
  if (!macros || typeof macros !== "object" || Array.isArray(macros)) {
    return {};
  }
  return macros;
}

async function mountEmbeddedModels(root) {
  const {mountModel} = await import(wasmMountRuntimeUrl());
  const modelRoots = root.matches?.("[data-model]")
    ? [root]
    : Array.from(root.querySelectorAll("[data-model]"));

  await Promise.all(modelRoots.map(async modelRoot => {
    const model = (modelRoot.dataset.model || "").trim();
    if (!model) {
      modelRoot.textContent = "missing data-model";
      return;
    }

    try {
      await mountModel(modelRoot, {
        model,
        code: extractPlainScript(modelRoot, "default-code"),
        ainput: extractPlainScript(modelRoot, "default-ainput"),
        rinput: extractPlainScript(modelRoot, "default-rinput"),
        bundleBaseUrl: "/wasm_bundle/",
        assetBaseUrl: "/wasm_bundle/",
      });
    } catch (error) {
      modelRoot.textContent = `failed to load ${model}: ${error}`;
    }
  }));
}

function wasmMountRuntimeUrl() {
  return "/wasm_bundle/script.js";
}

function extractPlainScript(root, className) {
  const el = root.querySelector(`script.${className}[type="text/plain"]`);
  if (!el) {
    return "";
  }
  return el.textContent || "";
}

function remarkComputationExtensions() {
  return tree => {
    transformNode(tree);
  };
}

function transformNode(node) {
  if (!node || !Array.isArray(node.children)) {
    return;
  }

  if (node.type === "code" || node.type === "inlineCode") {
    return;
  }

  const nextChildren = [];
  for (const child of node.children) {
    if (child.type === "paragraph") {
      const definitionList = transformDefinitionListParagraph(child);
      if (definitionList) {
        nextChildren.push(definitionList);
        continue;
      }
    }

    if (child.type === "blockquote") {
      nextChildren.push(transformBlockquoteNode(child));
      continue;
    }

    if (child.type === "text") {
      nextChildren.push(...splitMathTextNode(child));
      continue;
    }

    transformNode(child);
    nextChildren.push(child);
  }

  node.children = nextChildren;
  mergeAdjacentDefinitionLists(node);
}

function transformBlockquoteNode(node) {
  if (!Array.isArray(node.children) || node.children.length === 0) {
    return node;
  }

  const [firstChild, ...restChildren] = node.children;
  if (firstChild.type !== "paragraph" || !Array.isArray(firstChild.children) || firstChild.children.length === 0) {
    transformNode(node);
    return node;
  }

  const marker = parseAlertMarker(firstChild.children[0]);
  if (!marker) {
    transformNode(node);
    return node;
  }

  const nextFirstParagraph = {
    ...firstChild,
    children: trimLeadingWhitespace([
      ...marker.remainingChildren,
      ...firstChild.children.slice(1),
    ]),
  };

  const nextChildren = [];
  if (nextFirstParagraph.children.length > 0) {
    nextChildren.push(nextFirstParagraph);
  }
  nextChildren.push(...restChildren);

  const alertNode = {
    type: "alert",
    alertType: marker.alertType,
    children: nextChildren,
  };
  transformNode(alertNode);
  return alertNode;
}

function parseAlertMarker(node) {
  if (!node || node.type !== "text") {
    return null;
  }

  const match = /^(?:\s*)\[!([A-Za-z]+)\](?:\s+|$)/i.exec(node.value || "");
  if (!match) {
    return null;
  }

  const [, alertType] = match;
  const remainder = node.value.slice(match[0].length);
  const remainingChildren = remainder ? [{type: "text", value: remainder}] : [];
  return {
    alertType,
    remainingChildren,
  };
}

function trimLeadingWhitespace(children) {
  if (children.length === 0) {
    return children;
  }

  const [firstChild, ...restChildren] = children;
  if (firstChild.type !== "text") {
    return children;
  }

  const trimmedValue = firstChild.value.replace(/^\s+/, "");
  if (trimmedValue.length === 0) {
    return restChildren;
  }

  return [{...firstChild, value: trimmedValue}, ...restChildren];
}

function splitMathTextNode(node) {
  const text = node.value || "";
  const pieces = [];
  let index = 0;

  while (index < text.length) {
    const match = findNextMath(text, index);
    if (!match) {
      pieces.push(makeTextNode(text.slice(index)));
      break;
    }

    if (match.start > index) {
      pieces.push(makeTextNode(text.slice(index, match.start)));
    }

    if (match.error) {
      pieces.push({
        type: "mathError",
        value: text.slice(match.start),
        message: match.error,
      });
      return pieces.filter(isNonEmptyNode);
    }

    pieces.push({
      type: match.display ? "displayMath" : "inlineMath",
      value: match.content,
    });
    index = match.end;
  }

  return pieces.filter(isNonEmptyNode);
}

function transformDefinitionListParagraph(node) {
  const lines = splitInlineChildrenByNewline(node.children || []);
  if (lines.length < 2) {
    return null;
  }

  const termChildren = trimTrailingWhitespace(lineChildrenWithoutTrailingBreak(lines[0]));
  if (termChildren.length === 0) {
    return null;
  }

  const definitions = [];
  let currentDefinition = null;
  for (const line of lines.slice(1)) {
    const marker = extractDefinitionLine(line);
    if (marker) {
      currentDefinition = {
        type: "paragraph",
        children: trimTrailingWhitespace(trimLeadingWhitespace(marker.children)),
      };
      definitions.push(currentDefinition);
      continue;
    }

    if (!currentDefinition) {
      return null;
    }

    if (currentDefinition.children.length > 0) {
      currentDefinition.children.push(makeTextNode("\n"));
    }
    currentDefinition.children.push(...line);
    currentDefinition.children = trimTrailingWhitespace(currentDefinition.children);
  }

  if (definitions.length === 0 || definitions.some(definition => definition.children.length === 0)) {
    return null;
  }

  const termParagraph = {type: "paragraph", children: termChildren};
  transformNode(termParagraph);
  for (const definition of definitions) {
    transformNode(definition);
  }

  return {
    type: "definitionList",
    children: [{
      type: "definitionItem",
      termChildren: termParagraph.children,
      definitions,
    }],
  };
}

function splitInlineChildrenByNewline(children) {
  const lines = [[]];

  for (const child of children) {
    if (child.type !== "text") {
      lines.at(-1).push(child);
      continue;
    }

    const parts = child.value.split("\n");
    parts.forEach((part, index) => {
      if (part.length > 0) {
        lines.at(-1).push({...child, value: part});
      }
      if (index < parts.length - 1) {
        lines.push([]);
      }
    });
  }

  return lines;
}

function extractDefinitionLine(children) {
  if (children.length === 0) {
    return null;
  }

  const [firstChild, ...restChildren] = children;
  if (firstChild.type !== "text") {
    return null;
  }

  const match = /^(\s*):(?:[ \t]+|$)/.exec(firstChild.value || "");
  if (!match) {
    return null;
  }

  const remainder = firstChild.value.slice(match[0].length);
  const nextChildren = remainder.length > 0
    ? [{...firstChild, value: remainder}, ...restChildren]
    : restChildren;

  return {
    children: nextChildren,
  };
}

function lineChildrenWithoutTrailingBreak(children) {
  return children.filter(child => child.type !== "text" || child.value.length > 0);
}

function trimTrailingWhitespace(children) {
  if (children.length === 0) {
    return children;
  }

  const lastIndex = children.length - 1;
  const lastChild = children[lastIndex];
  if (lastChild.type !== "text") {
    return children;
  }

  const trimmedValue = lastChild.value.replace(/\s+$/, "");
  if (trimmedValue.length === 0) {
    return children.slice(0, lastIndex);
  }

  return [...children.slice(0, lastIndex), {...lastChild, value: trimmedValue}];
}

function mergeAdjacentDefinitionLists(node) {
  if (!Array.isArray(node.children) || node.children.length < 2) {
    return;
  }

  const mergedChildren = [];
  for (const child of node.children) {
    const previous = mergedChildren.at(-1);
    if (previous?.type === "definitionList" && child.type === "definitionList") {
      previous.children.push(...child.children);
      continue;
    }
    mergedChildren.push(child);
  }
  node.children = mergedChildren;
}

function definitionItemToHast(state, item) {
  return [
    {
      type: "element",
      tagName: "dt",
      properties: {},
      children: state.all({type: "paragraph", children: item.termChildren}),
    },
    ...item.definitions.map(definition => ({
      type: "element",
      tagName: "dd",
      properties: {},
      children: state.all({type: "root", children: [definition]}),
    })),
  ];
}

function findNextMath(text, fromIndex) {
  const inlineStart = text.indexOf(INLINE_OPEN, fromIndex);
  const displayStart = text.indexOf(DISPLAY_OPEN, fromIndex);

  let start = -1;
  let open = "";
  let close = "";
  let display = false;

  if (inlineStart !== -1 && (displayStart === -1 || inlineStart < displayStart)) {
    start = inlineStart;
    open = INLINE_OPEN;
    close = INLINE_CLOSE;
    display = false;
  } else if (displayStart !== -1) {
    start = displayStart;
    open = DISPLAY_OPEN;
    close = DISPLAY_CLOSE;
    display = true;
  }

  if (start === -1) {
    return null;
  }

  const contentStart = start + open.length;
  const closeIndex = text.indexOf(close, contentStart);
  if (closeIndex === -1) {
    return {
      start,
      error: `missing closing delimiter for ${open}`,
    };
  }

  return {
    start,
    end: closeIndex + close.length,
    display,
    content: text.slice(contentStart, closeIndex),
  };
}

function escapeMathDelimiters(text) {
  return text
    .replaceAll("\\\\", "\\\\\\\\")
    .replaceAll(INLINE_OPEN, "\\\\(")
    .replaceAll(INLINE_CLOSE, "\\\\)")
    .replaceAll(DISPLAY_OPEN, "\\\\[")
    .replaceAll(DISPLAY_CLOSE, "\\\\]");
}

function makeTextNode(value) {
  return {
    type: "text",
    value,
  };
}

function isNonEmptyNode(node) {
  return node.type !== "text" || node.value.length > 0;
}

function renderKatexNode(node, macros, displayMode) {
  try {
    return katex.renderToString(node.value, {
      displayMode,
      throwOnError: true,
      macros,
      output: "html",
    });
  } catch (error) {
    return renderMathError(node.value, String(error), displayMode);
  }
}

function renderMathError(source, message, displayMode) {
  const tagName = displayMode ? "div" : "span";
  return `<${tagName} class="md-math-error"><span class="md-math-error-label">KaTeX Error</span><code>${escapeHtml(source)}</code><span class="md-math-error-message">${escapeHtml(message)}</span></${tagName}>`;
}

function normalizeAlertType(value) {
  const normalized = String(value || "note").trim().toLowerCase();
  switch (normalized) {
    case "note":
    case "tip":
    case "important":
    case "warning":
    case "caution":
      return normalized;
    default:
      return "note";
  }
}

function alertLabel(type) {
  switch (type) {
    case "tip":
      return "Tip";
    case "important":
      return "Important";
    case "warning":
      return "Warning";
    case "caution":
      return "Caution";
    default:
      return "Note";
  }
}

function resolveMarkdownHref(href, basePath) {
  if (!href || isExternalHref(href) || href.startsWith("#")) {
    return href;
  }

  const resolvedPath = resolveRepositoryPath(href, basePath);
  if (resolvedPath.endsWith(".md")) {
    return `./md_preview.html?path=${encodeURIComponent(resolvedPath)}`;
  }

  return `/${resolvedPath}`;
}

function resolveAssetHref(href, basePath) {
  if (!href || isExternalHref(href) || href.startsWith("#")) {
    return href;
  }

  return `/${resolveRepositoryPath(href, basePath)}`;
}

function resolveRepositoryPath(target, basePath) {
  const cleanTarget = target.replace(/^\/+/, "");
  if (target.startsWith("/")) {
    return cleanTarget;
  }

  const baseDir = basePath.includes("/") ? basePath.slice(0, basePath.lastIndexOf("/") + 1) : "";
  const joined = `${baseDir}${cleanTarget}`;
  const normalized = [];
  for (const segment of joined.split("/")) {
    if (!segment || segment === ".") {
      continue;
    }
    if (segment === "..") {
      normalized.pop();
      continue;
    }
    normalized.push(segment);
  }
  return normalized.join("/");
}

function isExternalHref(href) {
  return /^(?:[a-z]+:)?\/\//i.test(href) || href.startsWith("mailto:") || href.startsWith("tel:");
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

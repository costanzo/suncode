import DOMPurify from "dompurify";
import katex from "katex";
import { Marked, type MarkedExtension, type Tokens } from "marked";
import markedShiki from "marked-shiki";
import { codeToHtml } from "shiki";

import "katex/dist/katex.min.css";

const escapeAttribute = (value: string) =>
  value.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

const sanitizeConfig = {
  USE_PROFILES: { html: true, mathMl: true },
  SANITIZE_NAMED_PROPS: true,
  FORBID_TAGS: ["style"],
  FORBID_CONTENTS: ["style", "script"],
  ADD_TAGS: ["svg", "path"],
  ADD_ATTR: ["d", "viewBox", "preserveAspectRatio", "xmlns", "target"],
};

if (typeof window !== "undefined" && DOMPurify.isSupported) {
  DOMPurify.addHook("afterSanitizeAttributes", (node: Element) => {
    if (!(node instanceof HTMLAnchorElement) || node.target !== "_blank") return;
    const rel = new Set((node.getAttribute("rel") ?? "").split(/\s+/).filter(Boolean));
    rel.add("noopener");
    rel.add("noreferrer");
    node.setAttribute("rel", Array.from(rel).join(" "));
  });
}

export async function renderMarkdown(text: string): Promise<string> {
  const html = await parser.parse(stripWrappedMarkup(text));
  return DOMPurify.isSupported ? DOMPurify.sanitize(html, sanitizeConfig) : "";
}

function stripWrappedMarkup(text: string): string {
  const wrapped = /^\s*<([A-Za-z]\w*)>\s*([\s\S]*?)\s*<\/\1>\s*$/;
  const match = text.match(wrapped);
  return match ? match[2] : text;
}

const inlineMathRegex = /^\\\(((?:\\.|[^\\\n])*?)\\\)/;
const blockMathRegex = /^\$\$\n([\s\S]+?)\n\$\$(?:\n|$)/;

const katexExtension: MarkedExtension = {
  extensions: [
    {
      name: "inlineKatex",
      level: "inline",
      start(source) {
        const index = source.indexOf("\\(");
        return index === -1 ? undefined : index;
      },
      tokenizer(source) {
        const match = source.match(inlineMathRegex);
        if (!match) return;
        return {
          type: "inlineKatex",
          raw: match[0],
          text: match[1].trim(),
          displayMode: false,
        };
      },
      renderer: renderKatexToken,
    },
    {
      name: "blockKatex",
      level: "block",
      tokenizer(source) {
        const match = source.match(blockMathRegex);
        if (!match) return;
        return {
          type: "blockKatex",
          raw: match[0],
          text: match[1].trim(),
          displayMode: true,
        };
      },
      renderer: renderKatexToken,
    },
  ],
};

function renderKatexToken(token: Tokens.Generic) {
  return katex.renderToString(typeof token.text === "string" ? token.text : "", {
    displayMode: token.displayMode === true,
    throwOnError: false,
  });
}

const parser = new Marked(
  {
    renderer: {
      link({ href, title, text }: Tokens.Link) {
        const titleAttribute = title ? ` title="${escapeAttribute(title)}"` : "";
        return `<a href="${escapeAttribute(href)}"${titleAttribute} class="external-link" target="_blank" rel="noopener noreferrer">${text}</a>`;
      },
    },
  },
  katexExtension,
  markedShiki({
    highlight(code, language) {
      return codeToHtml(code, {
        lang: language || "text",
        themes: {
          light: "github-light",
          dark: "github-dark",
        },
      });
    },
  }),
);

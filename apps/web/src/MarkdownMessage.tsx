import { useEffect, useRef, useState } from "react";
import { renderMarkdown } from "./markdown";

interface MarkdownMessageProps {
  text: string;
}

export function MarkdownMessage({ text }: MarkdownMessageProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const [html, setHtml] = useState("");

  useEffect(() => {
    let cancelled = false;
    void renderMarkdown(text).then((nextHtml) => {
      if (!cancelled) setHtml(nextHtml);
    });
    return () => {
      cancelled = true;
    };
  }, [text]);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;

    const buttons: HTMLButtonElement[] = [];
    root.querySelectorAll<HTMLPreElement>("pre").forEach((pre) => {
      if (pre.querySelector(".markdown-copy-button")) return;
      const code = pre.querySelector("code");
      if (!code) return;

      const button = document.createElement("button");
      button.type = "button";
      button.className = "markdown-copy-button";
      button.setAttribute("aria-label", "Copy code");
      button.textContent = "Copy";
      button.addEventListener("click", () => {
        void navigator.clipboard.writeText(code.textContent ?? "").then(() => {
          button.textContent = "Copied";
          window.setTimeout(() => {
            button.textContent = "Copy";
          }, 1400);
        });
      });
      pre.appendChild(button);
      buttons.push(button);
    });

    return () => {
      buttons.forEach((button) => {
        button.replaceWith();
      });
    };
  }, [html]);

  return <div ref={rootRef} className="markdown-body" dangerouslySetInnerHTML={{ __html: html }} />;
}

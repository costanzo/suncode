import { useEffect, useState } from "react";
import setiFontUrl from "../../../assets/fonts/seti.woff";

const fileIconDefinitions = {
  default: {
    glyph: "\uE023",
    label: "Generic file",
    color: "default",
  },
  javascript: {
    glyph: "\uE051",
    label: "JavaScript file",
    color: "yellow",
  },
  typescript: {
    glyph: "\uE099",
    label: "TypeScript file",
    color: "blue",
  },
  react: {
    glyph: "\uE07D",
    label: "React file",
    color: "blue",
  },
  markdown: {
    glyph: "\uE060",
    label: "Markdown file",
    color: "blue",
  },
  info: {
    glyph: "\uE04D",
    label: "Information document",
    color: "blue",
  },
  json: {
    glyph: "\uE055",
    label: "JSON file",
    color: "yellow",
  },
  tsconfig: {
    glyph: "\uE097",
    label: "TypeScript configuration file",
    color: "blue",
  },
  yaml: {
    glyph: "\uE0A7",
    label: "YAML file",
    color: "purple",
  },
  html: {
    glyph: "\uE048",
    label: "HTML file",
    color: "orange",
  },
  css: {
    glyph: "\uE01D",
    label: "CSS file",
    color: "blue",
  },
  config: {
    glyph: "\uE019",
    label: "Configuration file",
    color: "muted",
  },
  rust: {
    glyph: "\uE082",
    label: "Rust file",
    color: "muted",
  },
  python: {
    glyph: "\uE07B",
    label: "Python file",
    color: "blue",
  },
  csharp: {
    glyph: "\uE00B",
    label: "C Sharp file",
    color: "blue",
  },
  shell: {
    glyph: "\uE089",
    label: "Shell script",
    color: "green",
  },
  docker: {
    glyph: "\uE025",
    label: "Docker file",
    color: "blue",
  },
  git: {
    glyph: "\uE034",
    label: "Git file",
    color: "ink",
  },
  image: {
    glyph: "\uE04C",
    label: "Image file",
    color: "purple",
  },
  svg: {
    glyph: "\uE091",
    label: "SVG file",
    color: "purple",
  },
  audio: {
    glyph: "\uE005",
    label: "Audio file",
    color: "purple",
  },
  video: {
    glyph: "\uE09B",
    label: "Video file",
    color: "pink",
  },
  pdf: {
    glyph: "\uE06D",
    label: "PDF document",
    color: "red",
  },
  database: {
    glyph: "\uE022",
    label: "Database file",
    color: "pink",
  },
  archive: {
    glyph: "\uE0A9",
    label: "Archive file",
    color: "muted",
  },
  license: {
    glyph: "\uE05A",
    label: "License file",
    color: "yellow",
  },
  todo: {
    glyph: "\uE096",
    label: "Todo file",
    color: "todo",
  },
};

const fallbackLabels = {
  default: "•",
  javascript: "JS",
  typescript: "TS",
  react: "{}",
  markdown: "MD",
  info: "i",
  json: "{}",
  tsconfig: "TS",
  yaml: "Y",
  html: "<>",
  css: "#",
  config: "•",
  rust: "R",
  python: "PY",
  csharp: "C#",
  shell: "$",
  docker: "D",
  git: "G",
  image: "I",
  svg: "S",
  audio: "A",
  video: "V",
  pdf: "PDF",
  database: "DB",
  archive: "Z",
  license: "L",
  todo: "✓",
};

let setiFontPromise;
let setiFontLoaded = false;

function hasSetiFont() {
  return setiFontLoaded;
}

function loadSetiFont() {
  if (hasSetiFont()) return Promise.resolve(true);
  if (typeof FontFace === "undefined" || typeof document === "undefined" || !document.fonts) {
    return Promise.resolve(false);
  }

  setiFontPromise ??= new FontFace("Seti", `url(${setiFontUrl})`, {
    style: "normal",
    weight: "400",
  })
    .load()
    .then((font) => {
      document.fonts.add(font);
      setiFontLoaded = true;
      return true;
    })
    .catch(() => false);

  return setiFontPromise;
}

function useSetiFont() {
  const [isReady, setIsReady] = useState(hasSetiFont);

  useEffect(() => {
    let active = true;
    loadSetiFont().then((loaded) => {
      if (active && loaded) setIsReady(true);
    });

    return () => {
      active = false;
    };
  }, []);

  return isReady;
}

export const fileIconTypes = Object.keys(fileIconDefinitions);

export function FileIcon({ type = "default", size = 16, label, className = "" }) {
  const definition = fileIconDefinitions[type] ?? fileIconDefinitions.default;
  const accessibleLabel = label ?? definition.label;
  const hasLoadedFont = useSetiFont();
  const fallbackLabel = fallbackLabels[type] ?? fallbackLabels.default;

  return (
    <span
      className={`file-icon file-icon-${definition.color} ${hasLoadedFont ? "is-seti" : "is-fallback"} ${className}`}
      style={{ "--file-icon-size": `${size}px` }}
      aria-hidden={label ? undefined : true}
      aria-label={label ? accessibleLabel : undefined}
      role={label ? "img" : undefined}
    >
      {hasLoadedFont ? definition.glyph : fallbackLabel}
    </span>
  );
}

export function getFileIconDefinition(type) {
  return fileIconDefinitions[type] ?? fileIconDefinitions.default;
}

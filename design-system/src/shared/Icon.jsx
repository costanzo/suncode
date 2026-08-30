const paths = {
  home: (
    <>
      <path d="M3 10.5 12 3l9 7.5" />
      <path d="M5.5 9.5V21h13V9.5" />
      <path d="M9 21v-7h6v7" />
    </>
  ),
  foundation: (
    <>
      <path d="M4 5h16v14H4z" />
      <path d="M8 5v14M16 5v14M4 10h16M4 15h16" />
    </>
  ),
  assets: (
    <>
      <path d="M4 7h6l2-2h8v14H4z" />
      <path d="m7 16 3-3 2 2 2.5-3 2.5 4" />
    </>
  ),
  components: (
    <>
      <rect x="4" y="4" width="6" height="6" rx="1" />
      <rect x="14" y="4" width="6" height="6" rx="1" />
      <rect x="4" y="14" width="6" height="6" rx="1" />
      <rect x="14" y="14" width="6" height="6" rx="1" />
    </>
  ),
  platform: (
    <>
      <rect x="3" y="4" width="18" height="13" rx="2" />
      <path d="M8 21h8M12 17v4" />
    </>
  ),
  mobile: (
    <>
      <rect x="7" y="2" width="10" height="20" rx="2" />
      <path d="M10 5h4M11 19h2" />
    </>
  ),
  terminal: (
    <>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="m7 9 3 3-3 3M13 15h4" />
    </>
  ),
  project: (
    <>
      <path d="M4 6h6l2 2h8v11H4z" />
      <path d="M4 6V4h6l2 2" />
    </>
  ),
  folder: <path d="M3 6h7l2 2h9v11H3z" />,
  dependencies: (
    <>
      <path d="M4 7h6l2 2h8v10H4z" />
      <path d="M7 4h5l2 2M8 12h8M8 15h5" />
    </>
  ),
  file: (
    <>
      <path d="M6 3h8l4 4v14H6z" />
      <path d="M14 3v5h4" />
    </>
  ),
  "file-code": (
    <>
      <path d="M6 3h8l4 4v14H6z" />
      <path d="M14 3v5h4M10 12l-2 2 2 2M14 12l2 2-2 2" />
    </>
  ),
  "file-markdown": (
    <>
      <path d="M6 3h8l4 4v14H6z" />
      <path d="M14 3v5h4M8 16v-4l2 2 2-2v4M14 16h2" />
    </>
  ),
  "file-config": (
    <>
      <path d="M6 3h8l4 4v14H6z" />
      <path d="M14 3v5h4M9 12h6M9 15h6M9 18h4" />
    </>
  ),
  "file-text": (
    <>
      <path d="M6 3h8l4 4v14H6z" />
      <path d="M14 3v5h4M9 12h6M9 15h6M9 18h3" />
    </>
  ),
  search: (
    <>
      <circle cx="11" cy="11" r="7" />
      <path d="m16.5 16.5 4 4" />
    </>
  ),
  sun: (
    <>
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M19.1 4.9l-1.4 1.4M6.3 17.7l-1.4 1.4" />
    </>
  ),
  moon: <path d="M20.5 15.2A8.5 8.5 0 0 1 8.8 3.5 9 9 0 1 0 20.5 15.2Z" />,
  menu: (
    <>
      <path d="M4 7h16M4 12h16M4 17h16" />
    </>
  ),
  close: (
    <>
      <path d="m6 6 12 12M18 6 6 18" />
    </>
  ),
  "window-minimize": <path d="M5 12h14" />,
  "window-maximize": <rect x="5" y="5" width="14" height="14" />,
  "chevron-right": <path d="m9 5 7 7-7 7" />,
  arrow: (
    <>
      <path d="M5 12h14M14 7l5 5-5 5" />
    </>
  ),
  "arrow-up": (
    <>
      <path d="M12 19V5M6 11l6-6 6 6" />
    </>
  ),
  check: <path d="m5 12 4 4L19 6" />,
  workspace: (
    <>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M8 4v16M8 9h13M13 9v11" />
    </>
  ),
  files: (
    <>
      <path d="M4 5h6l2 2h8v12H4z" />
      <path d="M8 11h8M8 15h6" />
    </>
  ),
  git: (
    <>
      <circle cx="6" cy="5" r="2.5" />
      <circle cx="6" cy="19" r="2.5" />
      <circle cx="18" cy="5" r="2.5" />
      <path d="M6 7.5v9M8.5 17c5.25-.62 8.38-3.75 9-9" strokeWidth="1.8" />
    </>
  ),
  activity: (
    <>
      <path d="M3 12h4l2-6 4 12 2-6h6" />
    </>
  ),
  settings: (
    <>
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1-2.8 2.8-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.6v.2h-4V21a1.7 1.7 0 0 0-1-1.6 1.7 1.7 0 0 0-1.9.3l-.1.1L4.2 17l.1-.1a1.7 1.7 0 0 0 .3-1.9A1.7 1.7 0 0 0 3 14H2.8v-4H3a1.7 1.7 0 0 0 1.6-1 1.7 1.7 0 0 0-.3-1.9L4.2 7 7 4.2l.1.1a1.7 1.7 0 0 0 1.9.3A1.7 1.7 0 0 0 10 3v-.2h4V3a1.7 1.7 0 0 0 1 1.6 1.7 1.7 0 0 0 1.9-.3l.1-.1L19.8 7l-.1.1a1.7 1.7 0 0 0-.3 1.9 1.7 1.7 0 0 0 1.6 1h.2v4H21a1.7 1.7 0 0 0-1.6 1Z" />
    </>
  ),
  "panel-left": (
    <>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M9 4v16" />
    </>
  ),
  "panel-right": (
    <>
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M15 4v16" />
    </>
  ),
  plus: <path d="M12 5v14M5 12h14" />,
  copy: (
    <>
      <rect x="4" y="7" width="13" height="13" rx="2" />
      <path d="M8 7V5a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-2" />
    </>
  ),
  pin: (
    <>
      <path d="m9 3 6 6M10 8l-4 4 6 6 4-4M9 15l-6 6" />
    </>
  ),
  more: (
    <>
      <circle cx="5" cy="12" r="1" fill="currentColor" stroke="none" />
      <circle cx="12" cy="12" r="1" fill="currentColor" stroke="none" />
      <circle cx="19" cy="12" r="1" fill="currentColor" stroke="none" />
    </>
  ),
  refresh: (
    <>
      <path d="M20 11a8 8 0 0 0-14.7-4L3 10" />
      <path d="M3 4v6h6M4 13a8 8 0 0 0 14.7 4l2.3-3" />
      <path d="M21 20v-6h-6" />
    </>
  ),
};

export function Icon({ name, size = 18, className = "", ...props }) {
  return (
    <svg
      aria-hidden="true"
      className={`ds-icon ${className}`}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...props}
    >
      {paths[name] ?? paths.components}
    </svg>
  );
}

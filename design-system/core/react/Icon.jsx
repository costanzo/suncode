const paths = {
  home: <><path d="M3 10.5 12 3l9 7.5"/><path d="M5.5 9.5V21h13V9.5"/><path d="M9 21v-7h6v7"/></>,
  foundation: <><path d="M4 5h16v14H4z"/><path d="M8 5v14M16 5v14M4 10h16M4 15h16"/></>,
  assets: <><path d="M4 7h6l2-2h8v14H4z"/><path d="m7 16 3-3 2 2 2.5-3 2.5 4"/></>,
  components: <><rect x="4" y="4" width="6" height="6" rx="1"/><rect x="14" y="4" width="6" height="6" rx="1"/><rect x="4" y="14" width="6" height="6" rx="1"/><rect x="14" y="14" width="6" height="6" rx="1"/></>,
  platform: <><rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8M12 17v4"/></>,
  mobile: <><rect x="7" y="2" width="10" height="20" rx="2"/><path d="M10 5h4M11 19h2"/></>,
  terminal: <><rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9 3 3-3 3M13 15h4"/></>,
  project: <><path d="M4 6h6l2 2h8v11H4z"/><path d="M4 6V4h6l2 2"/></>,
  search: <><circle cx="11" cy="11" r="7"/><path d="m16.5 16.5 4 4"/></>,
  sun: <><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M19.1 4.9l-1.4 1.4M6.3 17.7l-1.4 1.4"/></>,
  moon: <path d="M20.5 15.2A8.5 8.5 0 0 1 8.8 3.5 9 9 0 1 0 20.5 15.2Z"/>,
  menu: <><path d="M4 7h16M4 12h16M4 17h16"/></>,
  close: <><path d="m6 6 12 12M18 6 6 18"/></>,
  arrow: <><path d="M5 12h14M14 7l5 5-5 5"/></>,
  check: <path d="m5 12 4 4L19 6"/>,
};

export function Icon({ name, size = 18, className = "", ...props }) {
  return (
    <svg aria-hidden="true" className={`ds-icon ${className}`} width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" {...props}>
      {paths[name] ?? paths.components}
    </svg>
  );
}

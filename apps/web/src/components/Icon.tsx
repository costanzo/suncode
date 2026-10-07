const paths = {
  folder: "M3 6h7l2 2h9v11H3z",
  chevron: "m9 5 7 7-7 7",
  monitor: "M3 4h18v13H3zM8 21h8M12 17v4",
  settings:
    "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8m8.5 4a8.5 8.5 0 0 0-.1-1.2l2-1.2-2-3.4-2.2 1a8 8 0 0 0-2.1-1.2L15.8 3h-4l-.3 2.4a8 8 0 0 0-2.1 1.2l-2.2-1-2 3.4 2 1.2A8.5 8.5 0 0 0 7 12c0 .4 0 .8.1 1.2l-2 1.2 2 3.4 2.2-1a8 8 0 0 0 2.1 1.2l.3 2.4h4l.3-2.4a8 8 0 0 0 2.1-1.2l2.2 1 2-3.4-2-1.2c.1-.4.2-.8.2-1.2",
  plus: "M12 5v14M5 12h14",
  send: "M5 12h14M13 6l6 6-6 6",
  lock: "M5 10h14v11H5zM8 10V7a4 4 0 0 1 8 0v3",
  panel: "M3 4h18v16H3zM15 4v16",
} as const;

export type IconName = keyof typeof paths;

export function Icon({ name }: { name: IconName }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d={paths[name]} />
    </svg>
  );
}

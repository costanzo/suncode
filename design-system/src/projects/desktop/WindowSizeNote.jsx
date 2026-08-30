export function WindowSizeNote({ width, height, minimumWidth, minimumHeight }) {
  return <div className="window-size-note" aria-label={`Default window size ${width} by ${height} DIP`}>
    <span>Default window size</span>
    <code>{width} × {height} DIP</code>
    {minimumWidth && minimumHeight && <small>Minimum {minimumWidth} × {minimumHeight} DIP</small>}
  </div>;
}

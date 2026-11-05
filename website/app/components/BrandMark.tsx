export function BrandMark({
  size = 24,
  label,
}: {
  size?: number;
  label?: string;
}) {
  const opticalSize = size <= 16 ? 16 : size <= 28 ? 24 : 40;
  return (
    <span
      className="brand-mark"
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
      style={{
        width: `var(--mark-size, ${size}px)`,
        height: `var(--mark-size, ${size}px)`,
        maskImage: `url(/suncode-mark-${opticalSize}-dark.svg)`,
      }}
    />
  );
}

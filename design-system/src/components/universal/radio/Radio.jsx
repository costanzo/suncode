export function Radio({ className = "", ...props }) {
  return <input type="radio" className={`radio-control ${className}`.trim()} {...props} />;
}

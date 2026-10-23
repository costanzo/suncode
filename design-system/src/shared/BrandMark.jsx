import markUrl from "../assets/logos/suncode-mark.svg";
import mark24Url from "../assets/logos/suncode-mark-24.svg";
import mark16Url from "../assets/logos/suncode-mark-16.svg";
import "./brand-mark.css";

export function BrandMark({ size = 24, className = "" }) {
  const url = size <= 16 ? mark16Url : size <= 24 ? mark24Url : markUrl;
  return (
    <i
      className={`suncode-brand-mark ${className}`}
      aria-hidden="true"
      style={{ width: size, height: size, "--brand-mark-image": `url("${url}")` }}
    />
  );
}

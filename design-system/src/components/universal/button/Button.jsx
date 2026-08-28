import { Icon } from "../../../shared/Icon.jsx";

export function Button({ children, variant = "neutral", size = "md", icon, className = "", ...props }) {
  const classes = ["btn", variant === "primary" ? "btn-primary" : variant === "danger" ? "btn-danger" : variant === "quiet" ? "btn-quiet" : "", size === "sm" ? "btn-sm" : size === "lg" ? "btn-lg" : "", className].filter(Boolean).join(" ");
  return <button className={classes} {...props}>{icon && <Icon name={icon} />}{children}</button>;
}

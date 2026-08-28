import { Icon } from "./Icon.jsx";
import { RouteLink } from "./PagePrimitives.jsx";

export function NotFoundPage({ path }) {
  return <div className="not-found"><Icon name="assets" size={28} /><h1>Module not found</h1><p><code>{path}</code> is not part of the current design catalog.</p><RouteLink to="/" className="btn btn-primary">Return to overview</RouteLink></div>;
}

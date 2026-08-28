import { Icon } from "./Icon.jsx";

export function RouteLink({ to, children, className = "", ...props }) {
  return <a href={`#${to}`} className={className} {...props}>{children}</a>;
}

export function Status({ tone = "neutral", children }) {
  return <span className={`module-status status-${tone}`}>{children}</span>;
}

export function PageHeader({ title, description, status, tone = "neutral" }) {
  return (
    <header className="page-header">
      <div className="page-header-copy">
        <div className="page-title-line">
          <h1>{title}</h1>
          {status && <Status tone={tone}>{status}</Status>}
        </div>
        <p>{description}</p>
      </div>
    </header>
  );
}

export function Section({ title, description, children, id, className = "" }) {
  return (
    <section className={`catalog-section ${className}`} id={id}>
      <div className="catalog-section-heading">
        <h2>{title}</h2>
        {description && <p>{description}</p>}
      </div>
      {children}
    </section>
  );
}

export function ModuleLink({ to, icon = "components", title, description, path, status, tone = "neutral" }) {
  return (
    <RouteLink to={to} className="module-link">
      <span className="module-link-icon"><Icon name={icon} /></span>
      <span className="module-link-copy">
        <span className="module-link-title"><strong>{title}</strong>{status && <Status tone={tone}>{status}</Status>}</span>
        <span>{description}</span>
        {path && <code>{path}</code>}
      </span>
      <Icon name="arrow" className="module-link-arrow" />
    </RouteLink>
  );
}

export function FileTree({ children }) {
  return <pre className="file-tree" aria-label="Directory structure">{children}</pre>;
}

export function DeferredPage({ title, platform, icon, description }) {
  return (
    <>
      <PageHeader title={title} description={description} path={`platforms/${platform}/`} status="Deferred" tone="deferred" />
      <Section id="boundary" title="Reserved adaptation boundary" description="The directory is intentional; implementation is not implied.">
        <div className="deferred-panel">
          <span className="deferred-icon"><Icon name={icon} size={28} /></span>
          <div>
            <h3>No Phase 1 component surface</h3>
            <p>This page records where future tokens, platform-only components, pages, and overrides belong. It does not turn a deferred client into a web mockup.</p>
          </div>
        </div>
      </Section>
      <Section id="ownership" title="Ownership contract">
        <FileTree>{`platforms/${platform}/\n├── index.jsx\n├── tokens/\n├── components/\n${platform === "mobile" ? "├── pages/\n├── styles/\n" : ""}└── overrides.css`}</FileTree>
      </Section>
    </>
  );
}

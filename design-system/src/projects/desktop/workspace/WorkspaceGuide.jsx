import { useState } from "react";
import { Icon } from "../../../shared/Icon.jsx";

const guideTabs = [
  ["actions", "User actions"],
  ["style", "Visual style"],
  ["logic", "Business logic"],
];

function makeGuideId(title) {
  return `workspace-guide-${title.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`;
}

export function WorkspaceGuideTitle({ title, open, onClick }) {
  return <button type="button" className="workspace-guide-title" aria-expanded={open} onClick={onClick}><h3>{title}</h3><Icon name="chevron-right" size={14} /></button>;
}

export function WorkspaceGuide({ title, tabs, side = "right", onClose }) {
  const [activeTab, setActiveTab] = useState("actions");
  const guideId = makeGuideId(title);
  return <aside className={`workspace-guide is-${side}`} aria-label={`${title} guide`}>
    <header className="workspace-guide-header"><div><span>MODULE GUIDE</span><h4>{title}</h4></div><button type="button" className="workspace-guide-close" aria-label={`Close ${title} guide`} title="Close guide" onClick={onClose}><Icon name="close" size={14} /></button></header>
    <div className="workspace-guide-tabs" role="tablist" aria-label={`${title} guide sections`}>{guideTabs.map(([id, label]) => <button key={id} type="button" role="tab" id={`${guideId}-${id}`} aria-selected={activeTab === id} aria-controls={`${guideId}-panel`} className={activeTab === id ? "is-active" : ""} onClick={() => setActiveTab(id)}>{label}</button>)}</div>
    <div className="workspace-guide-content" role="tabpanel" id={`${guideId}-panel`} aria-labelledby={`${guideId}-${activeTab}`}><ul>{(tabs[activeTab] ?? []).map((item) => <li key={item}>{item}</li>)}</ul></div>
  </aside>;
}

export function WorkspaceGuideState({ title, description, guide, side = "right", open, onToggle, onClose, className = "", children }) {
  return <div className={`workspace-guide-state ${open ? "is-guide-open" : ""} ${className}`}><WorkspaceGuideTitle title={title} open={open} onClick={onToggle} /><p>{description}</p>{open && <WorkspaceGuide title={guide.title ?? title} tabs={guide.tabs} side={guide.side ?? side} onClose={onClose} />}{children}</div>;
}

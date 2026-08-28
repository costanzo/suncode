import { useState } from "react";
import { Icon } from "../../../shared/Icon.jsx";
import { Specimen } from "../Specimen.jsx";

const tabs = ["Overview", "Files", "Provider trace"];

export function NavigationSpecimens() {
  const [tab, setTab] = useState("Overview");
  const [segment, setSegment] = useState("Changes");

  const handleTabKey = (event, index) => {
    let next = index;
    if (event.key === "ArrowRight") next = (index + 1) % tabs.length;
    else if (event.key === "ArrowLeft") next = (index - 1 + tabs.length) % tabs.length;
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = tabs.length - 1;
    else return;
    event.preventDefault();
    setTab(tabs[next]);
    document.getElementById(`component-tab-${next}`)?.focus();
  };

  return <div className="specimen-grid specimen-grid-3">
    <Specimen label="Project bay"><div className="nav-demo"><div className="nav-item active"><Icon name="assets" />Explorer</div><div className="nav-item"><Icon name="components" />Sessions<small>3</small></div><div className="nav-item"><Icon name="project" />Changes<small>8</small></div></div></Specimen>
    <Specimen label="Tabs"><div className="tabs" role="tablist" aria-label="Project details">{tabs.map((item, index) => <button id={`component-tab-${index}`} key={item} className={`tab ${tab === item ? "active" : ""}`} onClick={() => setTab(item)} onKeyDown={(event) => handleTabKey(event, index)} role="tab" aria-selected={tab === item} aria-controls="component-tabpanel" tabIndex={tab === item ? 0 : -1}>{item}</button>)}</div><div id="component-tabpanel" className="sample-note" role="tabpanel" aria-labelledby={`component-tab-${tabs.indexOf(tab)}`}>Selected: {tab}</div></Specimen>
    <Specimen label="Segmented view"><div className="segmented">{["Changes", "All files", "History"].map((item) => <button key={item} className={`segment ${segment === item ? "active" : ""}`} onClick={() => setSegment(item)}>{item}</button>)}</div><p className="sample-note">Selected: {segment}</p></Specimen>
  </div>;
}

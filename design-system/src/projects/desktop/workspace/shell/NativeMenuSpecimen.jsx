import { useState } from "react";
import { Icon } from "../../../../shared/Icon.jsx";
import { WindowsMenuBar } from "./WindowsMenuBar.jsx";

// Review fixtures only; the native Avalonia controller owns production commands.
export function NativeMenuSpecimen({ presentation = "reference" }) {
  const windows = presentation === "windows";
  const [platform, setPlatform] = useState(windows ? "Windows / Linux" : "macOS");
  const [context, setContext] = useState("active");
  const [selectedMenu, setSelectedMenu] = useState("View");
  const [left, setLeft] = useState("Sessions");
  const [right, setRight] = useState("Review");
  const [drawer, setDrawer] = useState("");
  const [navigation, setNavigation] = useState(true);
  const [model, setModel] = useState("gpt-5.6-sol");
  const [effort, setEffort] = useState("medium");
  const [feedback, setFeedback] = useState("");
  const mac = platform === "macOS";
  const command = mac ? "⌘" : "Ctrl+";
  const project = context !== "hub";
  const session = project && context !== "empty" && context !== "loading";
  const editable = session && context !== "archived";
  const idle = editable && !["running", "approval", "question"].includes(context);
  const row = (label, disabled = false, extra = {}) => ({ label, disabled, ...extra });
  const empty = (label) => [row(label, true)];
  const projects = context === "empty" ? empty("No Recent Projects") : [row("suncode", false, { checked: project, radio: true }), row("design-system")];
  const menus = {
    ...(mac ? { SunCode: [row("About SunCode"), null, row("Settings…", false, { shortcut: `${command},` })] } : {}),
    File: [row("Open Project…", false, { shortcut: `${command}O` }), row("Open Recent Project", false, { children: projects }), null,
      row("Back to Projects", !project), ...(!mac ? [null, row("Settings…", false, { shortcut: `${command},` })] : []), null,
      row("Close Window", false, { shortcut: `${command}W` })],
    Edit: [row("Undo", true, { shortcut: `${command}Z` }), row("Redo", true, { shortcut: mac ? "⇧⌘Z" : "Ctrl+Shift+Z" }), null,
      row("Cut", true, { shortcut: `${command}X` }), row("Copy", false, { shortcut: `${command}C` }), row("Paste", true, { shortcut: `${command}V` }), row("Select All", false, { shortcut: `${command}A` })],
    Navigate: [row("Projects", false, { children: projects }),
      row("Sessions", !project, { children: context === "empty" ? empty("No Sessions") : [row("Menu bar implementation", false, { checked: session && context !== "archived", radio: true }), row("Provider review")] }),
      row("Archived Sessions", !project, { children: context === "empty" ? empty("No Archived Sessions") : [row("Previous design review", false, { checked: context === "archived", radio: true })] }), null,
      row("Focus Conversation", !editable)],
    Session: [row("New Session…", !project || context === "loading", { shortcut: `${command}N` }), row("Rename Session…", !editable), row("Archive Session…", !idle),
      row("Restore Archived Session…", context !== "archived"), row("Delete Archived Session Permanently…", context !== "archived"), null, row("Retry Last Turn", !idle),
      row("Undo Agent Changes…", !idle, { children: context === "empty" ? empty("No Restorable Changes") : [row("turn-0198 — 2 files")] })],
    Agent: [row("Stop Current Turn", !["running", "approval", "question"].includes(context)), null,
      row("Model", !editable, { children: [row("OpenAI", false, { children: ["gpt-5.6-sol", "gpt-5.6"].map(label => row(label, !editable, { checked: model === label, radio: true, action: () => setModel(label) })) })] }),
      row("Reasoning Effort", !editable, { children: ["low", "medium", "high"].map(label => row(label, !editable, { checked: effort === label, radio: true, action: () => setEffort(label) })) }), null,
      row("Show Pending Approval", context !== "approval"), row("Show Pending Question", context !== "question")],
    View: [row("Sessions", !project, { checked: navigation && left === "Sessions", action: () => { setNavigation(!(navigation && left === "Sessions")); setLeft("Sessions"); } }),
      row("Explorer", !project, { checked: navigation && left === "Explorer", action: () => { setNavigation(!(navigation && left === "Explorer")); setLeft("Explorer"); } }),
      row("Project Navigation", !project, { checked: navigation, shortcut: `${command}1`, action: () => setNavigation(!navigation) }), null,
      ...["Built-in Browser", "Review", "Child Sessions"].map(label => row(label, !project || (label === "Child Sessions" && !session), { checked: right === label, action: () => setRight(right === label ? "" : label) })), null,
      ...(context === "no-git" ? [] : [row("Source Control", !project, { checked: drawer === "Source Control", shortcut: `${command}9`, action: () => setDrawer(drawer === "Source Control" ? "" : "Source Control") })]),
      ...["Provider Trace", "Tool Activity"].map(label => row(label, !session, { checked: drawer === label, action: () => setDrawer(drawer === label ? "" : label) }))],
    Window: [row("Minimize", false, { shortcut: `${command}M` }), row("Zoom"), row("Full Screen", false, { checked: false }), null,
      row("Merge All Windows", !project), row("Open Windows", false, { children: [row(project ? "suncode" : "SunCode", false, { checked: true, radio: true }), row("design-system")] })],
    Help: [row("Keyboard Shortcuts"), ...(!mac ? [row("About SunCode")] : [])],
  };
  const current = menus[selectedMenu] ? selectedMenu : "File";

  function renderItems(items, path = current) {
    return items.map((item, index) => {
      if (!item) return <div className="workspace-native-menu-divider" key={`${path}-${index}`} />;
      const key = `${path}-${item.label}`;
      const content = <><span className="workspace-native-menu-check" aria-hidden="true">{item.checked && <Icon name="check" size={12} />}</span><span className="workspace-native-menu-label">{item.label}</span>{item.shortcut && <span className="workspace-native-menu-shortcut">{item.shortcut}</span>}</>;
      if (item.children) return (
        <details className="workspace-native-menu-submenu" key={key}>
          <summary className="workspace-native-menu-item" aria-disabled={item.disabled} onClick={event => { if (item.disabled) event.preventDefault(); }}>
            {content}<Icon name="chevron-right" size={12} />
          </summary>
          <div className="workspace-native-menu-nested">{renderItems(item.children, key)}</div>
        </details>
      );
      return <button type="button" className="workspace-native-menu-item" key={key} disabled={item.disabled}
        aria-pressed={item.checked === undefined ? undefined : item.checked}
        onClick={() => { item.action?.(); setFeedback(`${item.label} selected in the review fixture.`); }}>{content}</button>;
    });
  }

  return (
    <div className="workspace-native-menu-specimen">
      <div className="workspace-native-menu-controls">
        {!windows && <label>Platform<select value={platform} onChange={event => setPlatform(event.target.value)}><option>macOS</option><option>Windows / Linux</option></select></label>}
        <label>Context<select value={context} onChange={event => { setContext(event.target.value); setFeedback(""); }}>
          <option value="active">Idle session</option><option value="running">Running turn</option><option value="approval">Pending approval</option><option value="question">Pending question</option><option value="archived">Archived session</option><option value="empty">Empty project</option><option value="hub">ProjectHub</option><option value="loading">Loading session</option><option value="no-git">Git unavailable</option>
        </select></label>
      </div>
      {windows ? <>
          <div className="windows-menu-scroll">
            <WindowsMenuBar
              menus={menus}
              context={context}
              onCommand={(item) => {
                item.action?.();
                setFeedback(`${item.label} selected in the review fixture.`);
              }}
            />
          </div>
          <div className="workspace-native-menu-notes">
            <p>
              Click the five-line button to reveal File, Edit, Navigate, Session, Agent, View,
              Window, and Help. Move across the labels to switch menus.
            </p>
            <p>
              The menu stays expanded when the pointer leaves it. Click elsewhere, choose a command,
              or press Escape to collapse it. Moving keyboard focus alone does not collapse it.
            </p>
            <p>
              Focus the preview and press F10 to open the menu. Use arrow keys to navigate, Enter to
              select, and Escape to return to the five-line button. At narrow widths, menu labels
              scroll horizontally while window controls stay fixed.
            </p>
            <p>
              Design reference only. Commands reuse the hierarchy below; confirmation dialogs and
              actual window actions belong to the desktop client.
            </p>
            <p role="status">{feedback}</p>
          </div>
      </> : <>
      <nav className="workspace-native-menu-bar" aria-label="Menu hierarchy">
        {Object.keys(menus).map(label => <button type="button" key={label} aria-pressed={current === label}
          className={current === label ? "is-open" : ""} onClick={() => setSelectedMenu(label)}>{label}</button>)}
      </nav>
      <div className="workspace-native-menu-review">
        <div className="workspace-native-menu-popover" aria-label={`${current} menu`} key={`${current}-${context}-${platform}`}>{renderItems(menus[current])}</div>
        <div className="workspace-native-menu-notes">
          <p>Review fixture. Select a menu to inspect its complete hierarchy; expand a submenu to inspect dynamic entries.</p>
          <p>View checks follow stored pane destinations. A constrained window may temporarily hide a checked pane. Source Control disappears when Git is unavailable.</p>
          <p>{mac ? "About and Settings belong to the macOS application menu. System lifecycle entries remain platform-owned." : "File owns Settings; Help owns About. This specimen records command placement; macOS owns the native global menu rendering."}</p>
          <p>Edit represents selected read-only text. Writable text controls enable their supported Undo, Redo, Cut, and Paste operations.</p>
          <p>Archive, restore, permanent delete, and checkpoint undo keep their existing confirmation dialogs. Pending-request commands navigate without approving or answering.</p>
          <p role="status">{feedback}</p>
        </div>
      </div>
      </>}
    </div>
  );
}

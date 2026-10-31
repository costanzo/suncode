import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Icon } from "../../../../shared/Icon.jsx";
import logoUrl from "../../../../assets/logos/suncode-logo-small.svg";
import "../../../../platforms/desktop/components/titlebar/titlebar.css";

const enabledItems = (element) => [...element.querySelectorAll(":scope > button:not(:disabled)")];
const focusFirst = (element) => enabledItems(element)[0]?.focus();

function MenuPanel({ items, label, onCommand, onClose, onSwitch, nested = false }) {
  const panelRef = useRef(null);
  const [child, setChild] = useState(null);
  const [flip, setFlip] = useState(false);

  useLayoutEffect(() => {
    if (!nested) return;
    const panel = panelRef.current;
    const frame = panel.closest(".windows-menu-preview").getBoundingClientRect();
    setFlip(panel.getBoundingClientRect().right > Math.min(frame.right, window.innerWidth));
  }, [nested]);

  function onKeyDown(event) {
    if (event.target.parentElement !== panelRef.current) return;
    const buttons = enabledItems(panelRef.current);
    const index = buttons.indexOf(event.target);
    const item = items[Number(event.target.dataset.index)];
    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const next =
        event.key === "Home"
          ? 0
          : event.key === "End"
            ? buttons.length - 1
            : (index + (event.key === "ArrowDown" ? 1 : -1) + buttons.length) % buttons.length;
      buttons[next]?.focus();
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      if (item?.children) {
        setChild(Number(event.target.dataset.index));
        requestAnimationFrame(() => focusFirst(panelRef.current.querySelector('[role="menu"]')));
      } else if (!nested) onSwitch(1);
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      if (nested) onClose();
      else onSwitch(-1);
    } else if (event.key === "Escape" && nested) {
      event.preventDefault();
      event.stopPropagation();
      onClose();
    } else if (event.key.length === 1 && !event.ctrlKey && !event.altKey && !event.metaKey) {
      const ordered = [...buttons.slice(index + 1), ...buttons.slice(0, index + 1)];
      ordered
        .find((button) =>
          button.textContent.trim().toLowerCase().startsWith(event.key.toLowerCase()),
        )
        ?.focus();
    }
  }

  return (
    <div
      ref={panelRef}
      role="menu"
      aria-label={label}
      className={`windows-menu-panel ${nested ? "is-nested" : ""} ${flip ? "opens-left" : ""}`}
      onKeyDown={onKeyDown}
    >
      {items.map((item, index) =>
        !item ? (
          <div key={index} role="separator" className="workspace-native-menu-divider" />
        ) : (
          <button
            key={item.label}
            type="button"
            data-index={index}
            role={
              item.checked === undefined
                ? "menuitem"
                : item.radio
                  ? "menuitemradio"
                  : "menuitemcheckbox"
            }
            aria-checked={item.checked}
            aria-haspopup={item.children ? "menu" : undefined}
            aria-expanded={item.children ? child === index : undefined}
            disabled={item.disabled}
            tabIndex={-1}
            className={`workspace-native-menu-item ${child === index ? "is-open" : ""}`}
            onPointerEnter={() => setChild(item.children && !item.disabled ? index : null)}
            onClick={() => {
              if (item.children) {
                setChild(index);
                requestAnimationFrame(() =>
                  focusFirst(panelRef.current.querySelector('[role="menu"]')),
                );
              } else onCommand(item);
            }}
          >
            <span className="workspace-native-menu-check" aria-hidden="true">
              {item.checked && <Icon name="check" size={12} />}
            </span>
            <span className="workspace-native-menu-label">{item.label}</span>
            {item.shortcut && (
              <span className="workspace-native-menu-shortcut">{item.shortcut}</span>
            )}
            {item.children && <Icon name="chevron-right" size={12} />}
          </button>
        ),
      )}
      {child !== null && items[child]?.children && (
        <div
          className="windows-menu-submenu-anchor"
          style={{
            top: panelRef.current?.querySelector(`[data-index="${child}"]`)?.offsetTop ?? 0,
          }}
        >
          <MenuPanel
            key={child}
            items={items[child].children}
            label={items[child].label}
            nested
            onCommand={onCommand}
            onSwitch={onSwitch}
            onClose={() => {
              panelRef.current.querySelector(`[data-index="${child}"]`)?.focus();
              setChild(null);
            }}
          />
        </div>
      )}
    </div>
  );
}

export function WindowsMenuBar({ menus, context, onCommand }) {
  const [expanded, setExpanded] = useState(false);
  const [selected, setSelected] = useState(null);
  const [offset, setOffset] = useState(0);
  const regionRef = useRef(null);
  const triggerRef = useRef(null);
  const labelsRef = useRef(null);
  const previousFocus = useRef(null);
  const labels = Object.keys(menus);

  function close(restore = false) {
    setExpanded(false);
    setSelected(null);
    if (restore) requestAnimationFrame(() => triggerRef.current?.focus());
  }

  function open(focus = false) {
    previousFocus.current = document.activeElement;
    setExpanded(true);
    setSelected(null);
    if (focus) requestAnimationFrame(() => labelsRef.current?.querySelector("button")?.focus());
  }

  function select(label, focus = false) {
    setSelected(label);
    if (focus)
      requestAnimationFrame(() => {
        const panel = regionRef.current?.querySelector('[role="menu"]');
        if (panel) focusFirst(panel);
      });
  }

  function switchMenu(step) {
    const next = labels[(labels.indexOf(selected) + step + labels.length) % labels.length];
    labelsRef.current
      ?.querySelector(`[data-menu="${next}"]`)
      ?.scrollIntoView({ block: "nearest", inline: "nearest" });
    select(next, true);
  }

  useEffect(() => {
    if (!expanded) return;
    const outside = (event) => {
      if (!regionRef.current?.contains(event.target)) close();
    };
    document.addEventListener("pointerdown", outside);
    const blur = () => close();
    window.addEventListener("blur", blur);
    return () => {
      document.removeEventListener("pointerdown", outside);
      window.removeEventListener("blur", blur);
    };
  }, [expanded]);

  useEffect(() => {
    close();
  }, [context]);

  useLayoutEffect(() => {
    if (!selected) return;
    const button = labelsRef.current?.querySelector(`[data-menu="${selected}"]`);
    const region = regionRef.current;
    if (button && region)
      setOffset(
        Math.max(
          0,
          Math.min(
            button.getBoundingClientRect().left - region.getBoundingClientRect().left,
            region.clientWidth - 320,
          ),
        ),
      );
  }, [selected]);

  return (
    <div
      className="windows-menu-preview"
      tabIndex={-1}
      aria-label="Windows project window preview"
      onKeyDownCapture={(event) => {
        if (!event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return;
        const shortcut = `Ctrl+${event.key.toUpperCase()}`;
        const item = Object.entries(menus)
          .filter(([group]) => group !== "Edit")
          .flatMap(([, items]) => items)
          .find((item) => item?.shortcut === shortcut && !item.disabled);
        if (!item) return;
        event.preventDefault();
        event.stopPropagation();
        onCommand(item);
        close();
      }}
      onKeyDown={(event) => {
        if (event.key === "F10" && !event.shiftKey) {
          event.preventDefault();
          if (expanded) close(true);
          else open(true);
        } else if (event.key === "Escape" && expanded) {
          event.preventDefault();
          close(true);
        }
      }}
    >
      <header className="windows-menu-titlebar">
        <img className="windows-menu-logo" src={logoUrl} alt="SunCode" />
        <div className="windows-menu-region" ref={regionRef}>
          {expanded ? (
            <nav
              ref={labelsRef}
              className="windows-menu-labels"
              role="menubar"
              aria-label="Windows application menu"
              onScroll={() => {
                if (selected) setSelected(null);
              }}
            >
              {labels.map((label, index) => (
                <button
                  key={label}
                  type="button"
                  role="menuitem"
                  data-menu={label}
                  tabIndex={selected === label || (!selected && index === 0) ? 0 : -1}
                  aria-haspopup="menu"
                  aria-expanded={selected === label}
                  className={selected === label ? "is-open" : ""}
                  onPointerEnter={() => select(label)}
                  onClick={() => select(label, true)}
                  onKeyDown={(event) => {
                    if (["ArrowRight", "ArrowLeft", "Home", "End"].includes(event.key)) {
                      event.preventDefault();
                      const next =
                        event.key === "Home"
                          ? 0
                          : event.key === "End"
                            ? labels.length - 1
                            : (index + (event.key === "ArrowRight" ? 1 : -1) + labels.length) %
                              labels.length;
                      labelsRef.current.querySelector(`[data-menu="${labels[next]}"]`)?.focus();
                      select(labels[next]);
                    } else if (["ArrowDown", "Enter", " "].includes(event.key)) {
                      event.preventDefault();
                      select(label, true);
                    }
                  }}
                >
                  {label}
                </button>
              ))}
            </nav>
          ) : (
            <div className="windows-menu-rest">
              <button
                ref={triggerRef}
                type="button"
                className="windows-menu-trigger"
                aria-label="Open main menu"
                aria-expanded="false"
                aria-haspopup="menu"
                title="Main menu (F10)"
                onClick={() => open(true)}
              >
                <Icon name="menu-lines" size={22} />
              </button>
              <span className="windows-menu-project">
                <Icon name="project" size={16} />
                <strong>{context === "hub" ? "Projects" : "suncode"}</strong>
                <span>SunCode</span>
              </span>
            </div>
          )}
          {expanded && selected && (
            <div className="windows-menu-popup-anchor" style={{ left: offset }}>
              <MenuPanel
                key={`${selected}-${context}`}
                items={menus[selected]}
                label={`${selected} menu`}
                onSwitch={switchMenu}
                onClose={() => close(true)}
                onCommand={(item) => {
                  onCommand(item);
                  close();
                  requestAnimationFrame(() => {
                    if (previousFocus.current?.isConnected) previousFocus.current.focus();
                    else triggerRef.current?.focus();
                  });
                }}
              />
            </div>
          )}
        </div>
        <div className="windows-window-controls" aria-label="Window controls (design preview)">
          <button
            type="button"
            aria-label="Open settings"
            title="Settings (Ctrl+,)"
            onClick={() => {
              close();
              onCommand({ label: "Settings…" });
            }}
          >
            <Icon name="settings" size={14} />
          </button>
          <button type="button" aria-label="Minimize preview window" title="Minimize (preview)">
            <Icon name="window-minimize" size={12} />
          </button>
          <button type="button" aria-label="Maximize preview window" title="Maximize (preview)">
            <Icon name="window-maximize" size={12} />
          </button>
          <button
            type="button"
            className="windows-window-close"
            aria-label="Close preview window"
            title="Close (preview)"
          >
            <Icon name="close" size={12} />
          </button>
        </div>
      </header>
      <div className="windows-menu-client">
        <aside>
          <strong>Sessions</strong>
          <span className="windows-menu-session">
            <Icon name="message" size={16} />
            Windows menu design
          </span>
        </aside>
        <div className="windows-menu-conversation">
          <strong>Windows menu design</strong>
          <p>Project commands, within reach.</p>
          <label>
            Conversation
            <input placeholder="Ask SunCode to work on your project…" />
          </label>
        </div>
      </div>
    </div>
  );
}

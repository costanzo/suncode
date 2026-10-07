"use client";

import { Menu, X } from "lucide-react";
import { useState } from "react";

export function MobileNav() {
  const [open, setOpen] = useState(false);

  return (
    <div className="mobile-nav">
      <button className="icon-button" type="button" aria-expanded={open} aria-controls="mobile-menu" aria-label={open ? "Close menu" : "Open menu"} onClick={() => setOpen((value) => !value)}>
        {open ? <X size={19} strokeWidth={1.8} /> : <Menu size={19} strokeWidth={1.8} />}
      </button>
      {open ? (
        <div className="mobile-menu" id="mobile-menu">
          <a href="#why" onClick={() => setOpen(false)}>Why SunCode</a>
          <a href="#capabilities" onClick={() => setOpen(false)}>Capabilities</a>
          <a href="#workflow" onClick={() => setOpen(false)}>Workflow</a>
          <a className="mobile-menu-cta" href="https://github.com/costanzo/suncode" target="_blank" rel="noreferrer" onClick={() => setOpen(false)}>View on GitHub</a>
        </div>
      ) : null}
    </div>
  );
}

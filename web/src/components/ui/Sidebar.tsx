import type { ReactNode } from "react";

export type View =
  | "entities"
  | "graph"
  | "generation"
  | "docs"
  | "manifest"
  | "traceability"
  | "health"
  | "review";

interface NavItem {
  id: View;
  label: string;
  icon: ReactNode;
}

interface NavGroup {
  label: string;
  items: NavItem[];
}

const NAV_GROUPS: NavGroup[] = [
  {
    label: "Knowledge",
    items: [
      { id: "entities", label: "Entities", icon: <IconEntities /> },
      { id: "graph", label: "Relationships", icon: <IconGraph /> },
      { id: "docs", label: "Docs", icon: <IconDocs /> },
    ],
  },
  {
    label: "Build",
    items: [
      { id: "generation", label: "Generation", icon: <IconGeneration /> },
      { id: "manifest", label: "Manifest", icon: <IconManifest /> },
      { id: "traceability", label: "Traceability", icon: <IconTrace /> },
    ],
  },
  {
    label: "Operations",
    items: [
      { id: "health", label: "Health", icon: <IconHealth /> },
      { id: "review", label: "Review", icon: <IconReview /> },
    ],
  },
];

export function getViewLabel(view: View): string {
  for (const group of NAV_GROUPS) {
    for (const item of group.items) {
      if (item.id === view) return item.label;
    }
  }
  return view;
}

interface SidebarProps {
  view: View;
  onChange: (next: View) => void;
  open?: boolean;
  onClose?: () => void;
}

export function Sidebar({ view, onChange, open, onClose }: SidebarProps) {
  return (
    <>
      {open && (
        <div
          className="kn-sidebar__scrim"
          aria-hidden="true"
          onClick={onClose}
        />
      )}
      <aside
        className={`kn-sidebar${open ? " kn-sidebar--open" : ""}`}
        aria-label="Primary"
      >
        <div className="kn-sidebar__brand">
          <span className="kn-sidebar__brand-mark" aria-hidden="true">kn</span>
          <span className="kn-sidebar__brand-text">know-now</span>
        </div>
        <nav className="kn-sidebar__nav" aria-label="Main navigation">
          {NAV_GROUPS.map((group) => (
            <div className="kn-sidebar__group" key={group.label}>
              <div className="kn-sidebar__group-label">{group.label}</div>
              <ul className="kn-sidebar__list">
                {group.items.map((item) => {
                  const active = view === item.id;
                  return (
                    <li key={item.id}>
                      <button
                        type="button"
                        className={`kn-sidebar__item${active ? " kn-sidebar__item--active" : ""}`}
                        onClick={() => {
                          onChange(item.id);
                          onClose?.();
                        }}
                        aria-current={active ? "page" : undefined}
                      >
                        <span className="kn-sidebar__icon" aria-hidden="true">
                          {item.icon}
                        </span>
                        <span className="kn-sidebar__label">{item.label}</span>
                      </button>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </nav>
      </aside>
    </>
  );
}

const SVG_PROPS = {
  width: 18,
  height: 18,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 1.75,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
  "aria-hidden": true,
};

function IconEntities() {
  return (
    <svg {...SVG_PROPS}>
      <rect x="3" y="4" width="18" height="6" rx="1.5" />
      <rect x="3" y="14" width="18" height="6" rx="1.5" />
      <path d="M7 7h.01M7 17h.01" />
    </svg>
  );
}

function IconGraph() {
  return (
    <svg {...SVG_PROPS}>
      <circle cx="6" cy="6" r="2.5" />
      <circle cx="18" cy="6" r="2.5" />
      <circle cx="12" cy="18" r="2.5" />
      <path d="M8 7l8 0M7.5 8.5L11 15.5M16.5 8.5L13 15.5" />
    </svg>
  );
}

function IconDocs() {
  return (
    <svg {...SVG_PROPS}>
      <path d="M6 3h9l4 4v14H6z" />
      <path d="M14 3v5h5M9 13h7M9 17h7M9 9h3" />
    </svg>
  );
}

function IconGeneration() {
  return (
    <svg {...SVG_PROPS}>
      <path d="M12 3v3M12 18v3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M3 12h3M18 12h3M5.6 18.4l2.1-2.1M16.3 7.7l2.1-2.1" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
}

function IconManifest() {
  return (
    <svg {...SVG_PROPS}>
      <path d="M5 4h14v16H5z" />
      <path d="M9 8h6M9 12h6M9 16h4" />
    </svg>
  );
}

function IconTrace() {
  return (
    <svg {...SVG_PROPS}>
      <path d="M4 7h6M14 7h6M4 17h6M14 17h6" />
      <path d="M10 7l4 10M14 7l-4 10" />
      <circle cx="4" cy="7" r="1.2" />
      <circle cx="20" cy="7" r="1.2" />
      <circle cx="4" cy="17" r="1.2" />
      <circle cx="20" cy="17" r="1.2" />
    </svg>
  );
}

function IconHealth() {
  return (
    <svg {...SVG_PROPS}>
      <path d="M3 12h4l2-5 4 10 2-5h6" />
    </svg>
  );
}

function IconReview() {
  return (
    <svg {...SVG_PROPS}>
      <path d="M5 5h14v11H8l-3 3z" />
      <path d="M9 10l2 2 4-4" />
    </svg>
  );
}

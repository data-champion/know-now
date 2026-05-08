import type { ReactNode } from "react";

interface TopBarProps {
  title: string;
  actions?: ReactNode;
  onMenuToggle?: () => void;
}

export function TopBar({ title, actions, onMenuToggle }: TopBarProps) {
  return (
    <div className="kn-topbar">
      {onMenuToggle && (
        <button
          type="button"
          className="kn-topbar__menu"
          aria-label="Toggle navigation"
          onClick={onMenuToggle}
        >
          <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M4 6h16M4 12h16M4 18h16"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              fill="none"
            />
          </svg>
        </button>
      )}
      <h1 className="kn-topbar__title">{title}</h1>
      {actions && <div className="kn-topbar__actions">{actions}</div>}
    </div>
  );
}

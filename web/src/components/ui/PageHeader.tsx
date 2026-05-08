import type { ReactNode } from "react";

interface PageHeaderProps {
  title: string;
  description?: string;
  actions?: ReactNode;
}

export function PageHeader({ title, description, actions }: PageHeaderProps) {
  return (
    <header className="kn-page-header">
      <div className="kn-page-header__text">
        <h2 className="kn-page-header__title">{title}</h2>
        {description && <p className="kn-page-header__desc">{description}</p>}
      </div>
      {actions && <div className="kn-page-header__actions">{actions}</div>}
    </header>
  );
}

import type { ReactNode } from "react";

interface CardProps {
  title?: string;
  actions?: ReactNode;
  wide?: boolean;
  className?: string;
  children: ReactNode;
}

export function Card({ title, actions, wide, className, children }: CardProps) {
  const cls = [
    "kn-card",
    wide ? "kn-card--wide" : "",
    className ?? "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <section className={cls}>
      {(title || actions) && (
        <header className="kn-card__header">
          {title && <h3 className="kn-card__title">{title}</h3>}
          {actions && <div className="kn-card__actions">{actions}</div>}
        </header>
      )}
      <div className="kn-card__body">{children}</div>
    </section>
  );
}

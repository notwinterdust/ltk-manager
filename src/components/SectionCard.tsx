import { type ReactNode } from "react";
import { twMerge } from "tailwind-merge";

interface SectionCardProps {
  title: string;
  children: ReactNode;
  className?: string;
}

export function SectionCard({ title, children, className }: SectionCardProps) {
  return (
    <section
      className={twMerge(
        "rounded-xl border border-surface-700/50 bg-surface-900/80 p-5",
        className,
      )}
    >
      <h3 className="mb-4 text-lg font-medium text-surface-100">{title}</h3>
      {children}
    </section>
  );
}

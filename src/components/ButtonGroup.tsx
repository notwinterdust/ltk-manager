import type { ReactNode } from "react";
import { twMerge } from "tailwind-merge";

interface ButtonGroupProps {
  children: ReactNode;
  className?: string;
}

export function ButtonGroup({ children, className }: ButtonGroupProps) {
  return (
    <div
      className={twMerge(
        "inline-flex [&>*:focus-visible]:relative [&>*:focus-visible]:z-10 [&>*:hover]:relative [&>*:hover]:z-10 [&>*:not(:first-child)]:-ml-px [&>*:not(:first-child)]:rounded-l-none [&>*:not(:last-child)]:rounded-r-none",
        className,
      )}
    >
      {children}
    </div>
  );
}

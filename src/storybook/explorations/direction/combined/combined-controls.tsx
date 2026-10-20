// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn } from "../../../../lib/styling";

// The direction's controls, drawn rather than wired: these mock-ups judge the
// look and layout, and the real primitives take it on once it is chosen.

export function Btn({
  children,
  className,
  variant = "default",
}: {
  children: ReactNode;
  className?: string;
  variant?: "default" | "ghost" | "primary";
}) {
  return (
    <span
      className={cn(
        "flex h-7 shrink-0 items-center gap-1.5 rounded-lg px-2.5 text-[12px] font-medium whitespace-nowrap [&_svg]:size-3.5",
        variant === "default" && "combined-raised",
        variant === "primary" && "combined-primary",
        variant === "ghost" && "text-(--fg-2)",
        className,
      )}
    >
      {children}
    </span>
  );
}

export function IconBtn({
  children,
  className,
  isActive = false,
  isDisabled = false,
  label,
}: {
  children: ReactNode;
  label: string;
  className?: string;
  isActive?: boolean;
  isDisabled?: boolean;
}) {
  return (
    <span
      aria-label={label}
      className={cn(
        "flex size-7 shrink-0 items-center justify-center rounded-lg text-(--fg-2) [&_svg]:size-4",
        isActive && "combined-raised text-(--fg)",
        isDisabled && "text-(--fg-3) opacity-60",
        className,
      )}
      title={label}
    >
      {children}
    </span>
  );
}

export function Segmented({
  className,
  items,
  selected,
}: {
  items: ReactNode[];
  selected: number;
  className?: string;
}) {
  return (
    <span className={cn("flex rounded-lg bg-(--fill) p-0.5", className)}>
      {items.map((item, index) => (
        <span
          className={cn(
            "flex h-6 flex-1 items-center justify-center rounded-md px-2 text-[12px] text-(--fg-2) [&_svg]:size-3.5",
            index === selected && "combined-raised text-(--fg)",
          )}
          // A fixed row of segments: never reordered.
          // eslint-disable-next-line @eslint-react/no-array-index-key
          key={index}
        >
          {item}
        </span>
      ))}
    </span>
  );
}

export function Toggle({ isOn }: { isOn: boolean }) {
  return (
    <span
      className={cn(
        "flex h-[18px] w-[30px] shrink-0 items-center rounded-full p-0.5 transition-colors",
        isOn ? "combined-accent justify-end" : "bg-(--fill-2)",
      )}
    >
      {/* On, the knob is what sits on the accent: white in light, the deep
          brown on dark's lighter orange. */}
      <span
        className={cn(
          "size-[14px] rounded-full shadow-[0_1px_2px_rgb(0_0_0/25%)]",
          isOn ? "bg-(--on-accent)" : "bg-white",
        )}
      />
    </span>
  );
}

/** A labelled row of a tool panel: title leading, control trailing. A row
 * that differs from the default carries a dot; pressing it resets the row. */
export function Row({
  children,
  isChanged = false,
  label,
}: {
  children: ReactNode;
  label: string;
  isChanged?: boolean;
}) {
  return (
    <div className="flex min-h-7 items-center justify-between gap-3">
      <span className="flex items-center gap-1.5 text-[12px] text-(--fg-2)">
        {label}
        {isChanged ? (
          <span
            aria-label={`Reset ${label}`}
            className="size-1.5 rounded-full bg-(--accent)"
          />
        ) : null}
      </span>
      {children}
    </div>
  );
}

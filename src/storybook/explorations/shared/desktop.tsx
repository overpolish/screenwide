// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn } from "../../../lib/styling";

const WALLPAPER =
  "linear-gradient(135deg, #5b7cfa 0%, #9b6cf0 45%, #f08a8a 100%)";

/** The desktop behind everything: wallpaper and two app windows. */
export function Desktop({
  children,
  className,
}: {
  children?: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn("relative overflow-hidden", className)}
      dir="ltr"
      style={{ background: WALLPAPER }}
    >
      <FakeWindow className="start-[8%] top-[12%] h-[62%] w-[52%]" />
      <FakeWindow className="start-[44%] top-[30%] h-[56%] w-[46%]" />
      {children}
    </div>
  );
}

export function FakeWindow({ className }: { className?: string }) {
  return (
    <div
      className={cn(
        "absolute flex flex-col overflow-hidden rounded-[6%/9%] bg-white/90 shadow-xl",
        className,
      )}
    >
      <div className="h-[9%] min-h-1.5 bg-black/5" />
      <div className="flex grow gap-[4%] p-[5%]">
        <div className="w-1/4 rounded-sm bg-black/5" />
        <div className="flex grow flex-col gap-[6%]">
          <div className="h-[12%] w-2/3 rounded-sm bg-black/10" />
          <div className="h-[8%] rounded-sm bg-black/5" />
          <div className="h-[8%] w-5/6 rounded-sm bg-black/5" />
        </div>
      </div>
    </div>
  );
}

export type StillKind = "area" | "camera" | "display" | "window";

/** A live still of exactly what will be recorded. */
export function TargetStill({
  className,
  target,
}: {
  target: StillKind;
  className?: string;
}) {
  if (target === "camera") {
    return (
      <div
        className={cn("relative overflow-hidden", className)}
        style={{ background: "linear-gradient(180deg, #4b5563, #1f2937)" }}
      >
        <span className="absolute start-1/2 top-[18%] aspect-square w-[16%] -translate-x-1/2 rounded-full bg-[#e5b796] rtl:translate-x-1/2" />
        <span className="absolute start-1/2 top-[50%] h-[60%] w-[46%] -translate-x-1/2 rounded-t-full bg-[#2563eb] rtl:translate-x-1/2" />
      </div>
    );
  }
  // A window records without the desktop behind it.
  if (target === "window") {
    return (
      <div className={cn("relative bg-recess", className)}>
        <FakeWindow className="inset-[8%_14%] shadow-none" />
      </div>
    );
  }
  // An area records only the part of the desktop inside it: the same
  // rectangle the overlay's area story selects.
  if (target === "area") {
    return (
      <div className={cn("relative overflow-hidden", className)}>
        <Desktop className="absolute start-[-45%] top-[-45%] h-[250%] w-[250%]" />
      </div>
    );
  }
  return <Desktop className={className} />;
}

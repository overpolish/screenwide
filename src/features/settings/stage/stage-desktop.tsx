// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn } from "../../../lib/styling";

import { useWallpaperSource } from "./use-wallpaper-source";

// The desktop a picture drawn by the page sits on, such as the island
// hanging from the top of the screen or Glide at work. The screen's
// geometry runs left to right in every language.

export function StageDesktop({
  children,
  isOn = true,
}: {
  children?: ReactNode;
  /** Off, the desktop is drained of colour, as a tool switched off is. */
  isOn?: boolean;
}) {
  const source = useWallpaperSource();
  return (
    <div
      className={cn(
        "absolute inset-0 overflow-hidden transition-[filter,opacity]",
        !isOn && "opacity-50 grayscale",
      )}
      dir="ltr"
    >
      <img
        alt=""
        className="absolute inset-0 size-full object-cover"
        src={source}
      />
      {children}
    </div>
  );
}

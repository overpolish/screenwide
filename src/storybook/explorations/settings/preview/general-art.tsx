// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { PREVIEW_DISPLAYS } from "../../../../features/recording-controls/island/island-preview";
import { cn } from "../../../../lib/styling";

// Small pictures at the start of a General row: a screen the size of a
// thumbnail, showing what the row's switch changes on it.

function MiniScreen({ children }: { children: ReactNode }) {
  return (
    <span
      aria-hidden
      className="relative block h-10 w-16 overflow-hidden rounded-sm"
      dir="ltr"
    >
      <img
        alt=""
        className="absolute inset-0 size-full object-cover"
        src={PREVIEW_DISPLAYS[0].thumbnail}
      />
      {children}
    </span>
  );
}

/** Another app's window, and Screenwide's beside it: kept, or cut out of
 * the picture with only its outline left to say where it was. */
export function CaptureArt({ isIncluded }: { isIncluded: boolean }) {
  return (
    <MiniScreen>
      <span className="absolute start-1.5 top-2 h-6 w-7 rounded-[3px] bg-white/90 shadow-sm" />
      <span
        className={cn(
          "absolute end-1.5 top-3.5 h-5 w-6 rounded-[3px] transition-all",
          isIncluded
            ? "bg-[#1c1c1e] shadow-sm"
            : "border border-dashed border-white/80 bg-transparent",
        )}
      >
        <span
          className={cn(
            "absolute start-1 top-1 size-1 rounded-full bg-[#ff5f57] transition-opacity",
            isIncluded ? "opacity-100" : "opacity-0",
          )}
        />
      </span>
    </MiniScreen>
  );
}

/** The island at the top of the screen, there when Screenwide starts or
 * not. */
export function StartupArt({ isShown }: { isShown: boolean }) {
  return (
    <MiniScreen>
      <span
        className={cn(
          "absolute start-1/2 top-0.5 h-1.5 -translate-x-1/2 rounded-full bg-island transition-all",
          isShown ? "w-6 opacity-100" : "w-2 opacity-0",
        )}
      />
    </MiniScreen>
  );
}

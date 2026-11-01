// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { motionDurationCss } from "../../../lib/motion";
import { cn } from "../../../lib/styling";

// The editor's chrome round the picture (the tool column, the strip, the
// title bar's modes) while the Export island stands over it: out of reach,
// and set back, faded and softly out of focus, so the picture alone is in
// focus as the export's preview. It eases there and back, at once where
// motion is reduced.

export function ChromeSetBack({
  children,
  className,
  isSetBack,
  keepsPointer = false,
}: {
  children: ReactNode;
  isSetBack: boolean;
  className?: string;
  /** Still pressed while set back, for a part that answers the pointer in
   * its own way, such as the strip, whose playhead still scrubs. */
  keepsPointer?: boolean;
}) {
  return (
    <div
      className={cn(
        "transition-[opacity,filter] motion-reduce:transition-none",
        isSetBack && "chrome-set-back",
        className,
      )}
      inert={isSetBack && !keepsPointer}
      style={{ transitionDuration: motionDurationCss("state") }}
    >
      {children}
    </div>
  );
}

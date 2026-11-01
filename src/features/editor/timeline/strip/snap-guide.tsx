// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";

// Where a dragged end has snapped, a dashed line down through the strip to
// the edge it holds to, the same for every drag that snaps
// (`strip-snap.ts`). The caller places it: across, and from where to where.

export function SnapGuide({
  className,
  style,
}: {
  style: React.CSSProperties;
  className?: string;
}) {
  return (
    <span
      aria-hidden
      className={cn(
        "pointer-events-none absolute w-px -translate-x-1/2 border-s border-dashed border-fg-secondary",
        className,
      )}
      style={style}
    />
  );
}

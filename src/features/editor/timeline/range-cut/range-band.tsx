// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { placed, type Span, type TimelineScale } from "../timeline-scale";

// The stretch the scissors draw across the strip, over lulls and beats
// alike, while it is drawn and once chosen.

export function RangeBand({
  scale,
  span,
}: {
  scale: TimelineScale;
  span: Span;
}) {
  return (
    <span
      aria-hidden
      className="pointer-events-none absolute inset-y-0.5 z-20 rounded-part bg-selected ring-2 ring-primary"
      dir="ltr"
      style={placed(scale, span.start, span.end)}
    />
  );
}

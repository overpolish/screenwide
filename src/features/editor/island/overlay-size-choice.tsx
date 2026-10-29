// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { PillGroup } from "../../../primitives/pill-group/pill-group";
import {
  OVERLAY_SIZES,
  useOverlaySizeLabel,
} from "../tool-panels/placement-options";

import type { OverlaySize } from "../../../bindings/OverlaySize";

/** How large an overlay drawn over the whole recording is, such as the
 * cursor or the shortcuts: its glyph drawn larger for each size, as the
 * type sizes are a letter growing, each named by how large it draws. */
export function OverlaySizeChoice({
  "aria-label": label,
  glyph,
  onChange,
  value,
}: {
  "aria-label": string;
  /** The overlay's glyph, drawn `side` px square. */
  glyph: (side: number) => ReactNode;
  onChange: (size: OverlaySize) => void;
  value: OverlaySize;
}) {
  const sizeLabel = useOverlaySizeLabel();
  return (
    <PillGroup
      aria-label={label}
      items={OVERLAY_SIZES.map((size, step) => ({
        ariaLabel: sizeLabel(size),
        icon: glyph(8 + step * 2),
        id: size,
      }))}
      onSelectionChange={(size) => {
        onChange(size as OverlaySize);
      }}
      selected={value}
    />
  );
}

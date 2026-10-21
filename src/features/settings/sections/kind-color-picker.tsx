// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { ColorButton } from "../../../composites/color-button/color-button";
import { t } from "../../../i18n/i18n";
import { usePopupPanelPicker } from "../../popup-panel/use-popup-panel-picker";

/** The colour as a kind keeps it: `#rrggbb` in lower case, whatever the
 * Colours panel reports. */
const kindColor = (color: string) => color.slice(0, 7).toLowerCase();

/** The colour grid's own size: the palette's row, with its padding. */
const GRID_SIZE = { height: 120, width: 340 };

/** How long the colour rests before it is kept: a drag in the Colours panel
 * reports every step, and each is shown at once, but only where it settles
 * is saved. */
const SETTLE_MS = 400;

export type KindColorChange = { color: string; customColor?: string | null };

/**
 * A kind's colour as a swatch in its row, opening the shared colour grid
 * under it, as every colour in the app is chosen: the palette, the kind's own
 * colour kept beside it, and Custom for the system's Colours panel. Each
 * colour tried shows at once; the one it rests on is kept. Keeping a custom
 * colour makes it the kind's own, and forgetting it clears that.
 */
export function KindColorPicker({
  color,
  customColor,
  isDisabled,
  name,
  onChange,
  onPreview,
}: {
  color: string;
  customColor: string | null;
  /** The kind's name, which the swatch is labelled with. */
  name: string;
  /** A colour to keep. */
  onChange: (change: KindColorChange) => void;
  /** A colour being tried, shown until it settles. */
  onPreview: (change: KindColorChange) => void;
  isDisabled?: boolean;
}) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const settleRef = useRef<{
    change: KindColorChange;
    timer: ReturnType<typeof setTimeout>;
  } | null>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  const keep = () => {
    const pending = settleRef.current;
    if (!pending) return;
    clearTimeout(pending.timer);
    settleRef.current = null;
    onChangeRef.current(pending.change);
  };
  // A colour still settling when the row goes is kept rather than lost.
  useEffect(() => keep, []);

  const label = t("settings-moment-color", { moment: name });
  const picker = usePopupPanelPicker({
    content: {
      kind: "colors",
      savedColors: customColor ? [customColor] : [],
      value: color,
    },
    id: `moment-kind-color:${name}`,
    label,
    onAction: (action, value) => {
      onChange({
        color,
        customColor: action === "save" ? kindColor(value) : null,
      });
    },
    onPick: (value) => {
      const change = { color: kindColor(value) };
      onPreview(change);
      if (settleRef.current) clearTimeout(settleRef.current.timer);
      settleRef.current = { change, timer: setTimeout(keep, SETTLE_MS) };
    },
  });

  return (
    <ColorButton
      aria-expanded={picker.isOpen}
      aria-label={label}
      isDisabled={isDisabled}
      onPress={(event) => {
        const trigger = triggerRef.current;
        if (trigger)
          void picker.toggle(
            trigger,
            GRID_SIZE,
            ["keyboard", "virtual"].includes(event.pointerType),
          );
      }}
      ref={triggerRef}
      value={color}
    />
  );
}

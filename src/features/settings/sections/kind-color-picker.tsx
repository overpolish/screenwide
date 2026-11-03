// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, useEffect, useRef } from "react";

import { ColorButton } from "../../../composites/color-button/color-button";
import { ColorGrid } from "../../../composites/color-grid/color-grid";
import { t } from "../../../i18n/i18n";
import { SettingsReveal } from "../settings-reveal";

/** The colour as a kind keeps it: `#rrggbb` in lower case, whatever the
 * Colours panel reports. */
const kindColor = (color: string) => color.slice(0, 7).toLowerCase();

/** How long the colour rests before it is kept: a drag in the Colours panel
 * reports every step, and each is shown at once, but only where it settles
 * is saved. */
const SETTLE_MS = 400;

type KindColorChange = { color: string; customColor?: string | null };

type KindColorProps = {
  color: string;
  customColor: string | null;
  /** A colour to keep. */
  onChange: (change: KindColorChange) => void;
  /** A colour being tried, shown until it settles. */
  onPreview: (change: KindColorChange) => void;
  isDisabled?: boolean;
};

/** The shared colour grid, open under a kind's row. Each colour tried shows
 * at once; the one it rests on is kept, and one still settling when the grid
 * closes is kept rather than lost. */
function KindColorGrid({
  autoFocus,
  color,
  customColor,
  isDisabled,
  onChange,
  onClose,
  onPreview,
}: KindColorProps & { autoFocus: boolean; onClose: () => void }) {
  const gridRef = useRef<HTMLDivElement>(null);
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
  useEffect(() => keep, []);

  // Opened from the keyboard, the grid takes focus on the chosen swatch, as
  // the panel it replaces did; a click leaves it where the pointer is.
  useEffect(() => {
    if (!autoFocus) return;
    const grid = gridRef.current;
    grid
      ?.querySelector<HTMLElement>("[data-selected]")
      ?.focus({ preventScroll: true });
  }, [autoFocus]);

  return (
    <div
      className="self-start"
      onKeyDown={(event) => {
        if (event.key !== "Escape" || event.defaultPrevented) return;
        event.preventDefault();
        onClose();
      }}
      ref={gridRef}
    >
      <ColorGrid
        columns={10}
        isDisabled={isDisabled}
        offersCustom
        onChange={(value) => {
          const change = { color: kindColor(value) };
          onPreview(change);
          if (settleRef.current) clearTimeout(settleRef.current.timer);
          settleRef.current = { change, timer: setTimeout(keep, SETTLE_MS) };
        }}
        onRemoveColor={() => {
          onChange({ color, customColor: null });
        }}
        onSaveColor={(value) => {
          onChange({ color, customColor: kindColor(value) });
        }}
        savedColors={customColor ? [customColor] : []}
        value={color}
      />
    </div>
  );
}

/**
 * A kind's row led by its colour as a swatch, which opens the shared colour
 * grid under the row, as every colour in the app is chosen: the palette, the
 * kind's own colour kept beside it, and Custom for the system's Colours
 * panel. Keeping a custom colour makes it the kind's own, and forgetting it
 * clears that. Escape in the grid closes it and puts focus back on the
 * swatch.
 */
export function KindColorPicker({
  children,
  isOpen,
  name,
  onOpenChange,
  ...props
}: KindColorProps & {
  /** The rest of the kind's row, after the swatch. */
  children: ReactNode;
  isOpen: boolean;
  /** The kind's name, which the swatch is labelled with. */
  name: string;
  onOpenChange: (isOpen: boolean) => void;
}) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const openedByKeyboardRef = useRef(false);
  return (
    <div className="flex flex-col">
      <div className="flex items-center gap-inline">
        <ColorButton
          aria-expanded={isOpen}
          aria-label={t("settings-moment-color", { moment: name })}
          isDisabled={props.isDisabled}
          onPress={(event) => {
            openedByKeyboardRef.current = ["keyboard", "virtual"].includes(
              event.pointerType,
            );
            onOpenChange(!isOpen);
          }}
          ref={triggerRef}
          value={props.color}
        />
        {children}
      </div>
      <SettingsReveal className="flex pt-separate">
        {isOpen ? (
          <KindColorGrid
            {...props}
            autoFocus={openedByKeyboardRef.current}
            onClose={() => {
              onOpenChange(false);
              triggerRef.current?.focus();
            }}
          />
        ) : null}
      </SettingsReveal>
    </div>
  );
}

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { keyLabel } from "../../../../primitives/keyboard/key-label";
import { Keyboard, Shortcut } from "../../../../primitives/keyboard/keyboard";
import { type Span } from "../timeline-scale";

// A shortcut pressed while recording, drawn as a marker: its keys, and a
// short tick under them at the moment it was pressed, the same at any zoom.
// How long its keys show on the picture is not drawn: it is one setting for
// every shortcut, so it says nothing about this one. Placed by the
// recording, it is never moved or trimmed: pressed, it is chosen, and the
// Keyboard island it opens styles every shortcut at once; chosen, it can be
// deleted. Where shortcuts were pressed close together, the lane sets its
// keys a little after the tick (`labelOffset`), so they sit side by side;
// the tick, under the keys, never crosses another shortcut's.

export type TimelineShortcut = Span & {
  id: string;
  /** Its keys, in the order pressed, as the recording names them. */
  keys: string[];
};

export type ShortcutChipProps = {
  isSelected: boolean;
  /** Told a press, whose event says whether Shift takes it in or out. */
  onPress: (event: React.MouseEvent) => void;
  shortcut: TimelineShortcut;
  /** Where it was pressed, in px along the strip. */
  x: number;
  /** How far past the tick its keys sit, in px, clear of the shortcut
   * before. */
  labelOffset?: number;
  /** Told how wide its keys are drawn, in px, so the lane can set the next
   * shortcut's clear of them. */
  onMeasure?: (width: number) => void;
};

export function ShortcutChip({
  isSelected,
  labelOffset = 0,
  onMeasure,
  onPress,
  shortcut,
  x,
}: ShortcutChipProps) {
  const measure = useCallback(
    (element: HTMLButtonElement | null) => {
      if (!element || !onMeasure) return;
      const observer = new ResizeObserver(([entry]) => {
        onMeasure(entry.borderBoxSize[0].inlineSize);
      });
      observer.observe(element);
      return () => {
        observer.disconnect();
      };
    },
    [onMeasure],
  );
  return (
    <>
      <span
        aria-hidden
        className={cn(
          "pointer-events-none absolute -bottom-1 h-1.5 w-px -translate-x-1/2",
          isSelected ? "bg-primary" : "bg-fg-tertiary",
        )}
        style={{ insetInlineStart: x }}
      />
      <button
        aria-label={t("editor-timeline-shortcut", {
          keys: shortcut.keys.map(keyLabel).join("+"),
        })}
        aria-pressed={isSelected}
        className={cn(
          "absolute inset-y-0 flex w-max cursor-default items-center rounded-part px-0.5 outline-none transition-colors focus-visible:outline-2 focus-visible:outline-focus",
          isSelected
            ? "bg-selected ring-2 ring-primary ring-inset"
            : "hover:bg-fill",
        )}
        dir="ltr"
        onClick={onPress}
        onPointerDown={(event) => {
          event.stopPropagation();
        }}
        ref={measure}
        // Its keys start at its tick, or just after the shortcut before.
        style={{ insetInlineStart: x + labelOffset }}
      >
        <Shortcut aria-hidden>
          {shortcut.keys.map((key) => (
            <Keyboard key={key}>{key}</Keyboard>
          ))}
        </Shortcut>
      </button>
    </>
  );
}

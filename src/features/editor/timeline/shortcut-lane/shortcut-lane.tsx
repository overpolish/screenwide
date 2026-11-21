// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { use, useRef, useState } from "react";

import { isNear, StripNearContext } from "../strip/strip-near";
import { type TimelineScale } from "../timeline-scale";

import { ShortcutChip, type TimelineShortcut } from "./shortcut-chip";
import { placeShortcutKeys } from "./shortcut-placement";

// The shortcuts pressed while recording, a lane over the beats, each its
// keys with a tick under them where it was pressed, the keys set clear of
// each other and kept on the strip (`placeShortcutKeys`). Its tick stays
// where it was pressed.
// Pressing one chooses it; with Shift, it is taken in or left out of what
// is chosen. Delete or Backspace deletes what is chosen.

/** How wide, in px, keys are taken to be until they are drawn. */
const KEYS_PX = 40;

export type ShortcutLaneProps = {
  /** Told shortcuts to delete, by id. */
  onDelete: (ids: string[]) => void;
  /** Told what is chosen now, by id; empty lets the choice go. */
  onSelect: (ids: string[]) => void;
  scale: TimelineScale;
  /** The shortcuts chosen, by id. */
  selected: string[];
  shortcuts: TimelineShortcut[];
  /** The strip's width, in px. */
  width: number;
};

export function ShortcutLane({
  onDelete,
  onSelect,
  scale,
  selected,
  shortcuts,
  width,
}: ShortcutLaneProps) {
  const near = use(StripNearContext);
  // How wide each shortcut's keys are drawn, by id, as they report it.
  const [widths, setWidths] = useState<Record<string, number>>({});
  const measuresRef = useRef(new Map<string, (px: number) => void>());
  const measureOf = (id: string) => {
    let measure = measuresRef.current.get(id);
    if (!measure) {
      measure = (px) => {
        setWidths((current) =>
          current[id] === px ? current : { ...current, [id]: px },
        );
      };
      measuresRef.current.set(id, measure);
    }
    return measure;
  };
  const sorted = [...shortcuts].sort(
    (first, second) => first.start - second.start,
  );
  const ticks = sorted.map((shortcut) => ({
    keysPx: widths[shortcut.id] ?? KEYS_PX,
    x: scale.toX(shortcut.start) * width,
  }));
  const starts = placeShortcutKeys(ticks, width);
  const placed = sorted.map((shortcut, index) => ({
    from: starts[index],
    shortcut,
    x: ticks[index].x,
  }));

  return (
    <div
      className="relative h-5"
      dir="ltr"
      // The chosen chip has the focus, so its keys reach here.
      onKeyDown={(event) => {
        if (selected.length === 0) return;
        if (event.key !== "Delete" && event.key !== "Backspace") return;
        event.preventDefault();
        onDelete(selected);
      }}
    >
      {placed
        .filter(
          ({ shortcut }) =>
            selected.includes(shortcut.id) ||
            isNear(near, scale, { end: shortcut.start, start: shortcut.start }),
        )
        .map(({ from, shortcut, x }) => (
          <ShortcutChip
            isSelected={selected.includes(shortcut.id)}
            key={shortcut.id}
            labelOffset={from - x}
            onMeasure={measureOf(shortcut.id)}
            onPress={(event) => {
              const isIn = selected.includes(shortcut.id);
              onSelect(
                !event.shiftKey
                  ? [shortcut.id]
                  : isIn
                    ? selected.filter((id) => id !== shortcut.id)
                    : [...selected, shortcut.id],
              );
            }}
            shortcut={shortcut}
            x={x}
          />
        ))}
    </div>
  );
}

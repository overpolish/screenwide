// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { type TimelineScale } from "../timeline-scale";

import { ShortcutChip, type TimelineShortcut } from "./shortcut-chip";

// The shortcuts pressed while recording, a lane over the beats, each its
// keys with a tick under them where it was pressed. Pressed close together,
// a shortcut's keys are set after the keys before them, a clear gap apart,
// so they sit side by side as separate shortcuts rather than over each
// other or run together; its tick stays where it was pressed.
// Pressing one chooses it; with Shift, it is taken in or left out of what
// is chosen. Delete or Backspace deletes what is chosen.

/** The room, in px, kept between one shortcut's keys and the next's: wider
 * than the room between keys, so each reads as its own. */
const GAP_PX = 10;
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
  // Each one's tick where it was pressed, and its keys clear of the last.
  let reached = -Infinity;
  const placed = [...shortcuts]
    .sort((first, second) => first.start - second.start)
    .map((shortcut) => {
      const x = scale.toX(shortcut.start) * width;
      const labelOffset = Math.max(0, reached + GAP_PX - x);
      reached = x + labelOffset + (widths[shortcut.id] ?? KEYS_PX);
      return { labelOffset, shortcut, x };
    });

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
      {placed.map(({ labelOffset, shortcut, x }) => (
        <ShortcutChip
          isSelected={selected.includes(shortcut.id)}
          key={shortcut.id}
          labelOffset={labelOffset}
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

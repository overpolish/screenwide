// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import type { PopupPanelItem } from "./store";

// A list's toggles set by dragging: pressed, a toggle flips at once, and
// every other toggle of its section the pointer then passes over is set
// the same way, on where the first was turned on, off where it was turned
// off, so a run of them is set in one stroke. A toggle that keeps one of
// its section on (`keepsOneOn`) is skipped where it is the last one on.

/** The attribute a toggle's row carries its item's id in. */
export const TOGGLE_ATTRIBUTE = "data-toggle";

/** Whether `item` can be flipped with `ticks` on. */
const canFlip = (
  item: PopupPanelItem,
  items: readonly PopupPanelItem[],
  ticks: ReadonlySet<string>,
) => {
  if (!item.togglesInPlace) return false;
  if (!item.keepsOneOn || !ticks.has(item.id)) return true;
  return items.some(
    (other) =>
      other !== item &&
      other.keepsOneOn &&
      other.section === item.section &&
      ticks.has(other.id),
  );
};

export function useToggleDrag({
  items,
  onFlip,
  selectedIds,
}: {
  items: readonly PopupPanelItem[];
  /** Told a toggle flipped, with every id on once it has. */
  onFlip: (item: PopupPanelItem, selectedIds: string[]) => void;
  selectedIds: readonly string[];
}) {
  // The toggle a drag began on, whose press that follows on release is
  // already taken.
  const pressedRef = useRef<string | null>(null);
  const latestRef = useRef({ items, onFlip, selectedIds });
  latestRef.current = { items, onFlip, selectedIds };

  /** Flips `item` where it can be, keeping `ticks` in step; whether it did. */
  const flip = (item: PopupPanelItem, ticks: Set<string>) => {
    const { items: all, onFlip: told } = latestRef.current;
    if (!canFlip(item, all, ticks)) return false;
    if (ticks.has(item.id)) ticks.delete(item.id);
    else ticks.add(item.id);
    told(
      item,
      all.map((each) => each.id).filter((id) => ticks.has(id)),
    );
    return true;
  };

  const itemAt = (target: Element | null) => {
    const id = target
      ?.closest(`[${TOGGLE_ATTRIBUTE}]`)
      ?.getAttribute(TOGGLE_ATTRIBUTE);
    return latestRef.current.items.find((item) => item.id === id);
  };

  const onPointerDown = (event: React.PointerEvent) => {
    if (event.button !== 0) return;
    const first = itemAt(event.target as Element);
    if (!first) return;
    const ticks = new Set(latestRef.current.selectedIds);
    if (!flip(first, ticks)) return;
    const isOn = ticks.has(first.id);
    pressedRef.current = first.id;
    const onMove = (move: PointerEvent) => {
      const item = itemAt(
        document.elementFromPoint(move.clientX, move.clientY),
      );
      // Kept to the first one's section, so a drag down a list of formats
      // never reaches a toggle that means something else.
      if (item && item.section === first.section && ticks.has(item.id) !== isOn)
        flip(item, ticks);
    };
    const onEnd = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onEnd);
      window.removeEventListener("pointercancel", onEnd);
      // The press on release lands first, and is let go once it has.
      setTimeout(() => {
        pressedRef.current = null;
      });
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onEnd);
    window.addEventListener("pointercancel", onEnd);
  };

  return {
    /** Flips `item` for a press that no drag took, as a key's. */
    flipPressed: (item: PopupPanelItem) => {
      if (pressedRef.current === item.id) return;
      flip(item, new Set(latestRef.current.selectedIds));
    },
    onPointerDown,
  };
}

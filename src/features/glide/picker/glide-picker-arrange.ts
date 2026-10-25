// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A screen's centre on the desktop, in the desktop's own units. Spaces sit
 * in a row a unit apart. */
export type GlidePickerScreen = { x: number; y: number };

/** A screen's place in the picker, in px from its top start corner. */
type GlidePickerSlot = { x: number; y: number };

/**
 * Lays screens out the way the native cards are (`card_offsets`): every
 * slot the same size, keeping the screens' centres where they are relative to
 * one another, spread just enough to leave `gap` between each pair.
 */
export function arrangeSlots(
  screens: GlidePickerScreen[],
  size: { height: number; width: number },
  gap: number,
) {
  let scale = 0;
  screens.forEach((first, index) => {
    for (const second of screens.slice(index + 1)) {
      const dx = Math.abs(second.x - first.x);
      const dy = Math.abs(second.y - first.y);
      // Mirrored displays share a centre and a slot.
      if (dx + dy < 1) continue;
      scale = Math.max(
        scale,
        Math.min((size.width + gap) / dx, (size.height + gap) / dy),
      );
    }
  });
  const left = Math.min(...screens.map(({ x }) => x));
  const top = Math.min(...screens.map(({ y }) => y));
  const slots: GlidePickerSlot[] = screens.map(({ x, y }) => ({
    x: (x - left) * scale,
    y: (y - top) * scale,
  }));
  return {
    height: Math.max(...slots.map(({ y }) => y)) + size.height,
    slots,
    width: Math.max(...slots.map(({ x }) => x)) + size.width,
  };
}

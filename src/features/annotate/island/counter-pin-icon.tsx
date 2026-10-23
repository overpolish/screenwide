// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A counter as a filled pin pointing at twelve o'clock, its disc centred in
 * the box so it turns about the disc as the counter does. Lucide's
 * `MapPinPlusInside` outline, the counter tool's icon and the renderer's own
 * counter shape, turned half a turn and lifted two units, without its plus:
 * the plus names the tool, and here the pin only aims. */
export function CounterPinIcon({ className }: { className?: string }) {
  return (
    <svg
      aria-hidden
      className={className}
      fill="currentColor"
      viewBox="0 0 24 24"
    >
      <path d="M4 12c0-4.993 5.539-10.193 7.399-11.799a1 1 0 0 1 1.202 0C14.461 1.807 20 7.007 20 12a8 8 0 0 1-16 0" />
    </svg>
  );
}

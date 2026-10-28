// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Centres the chosen item in a scroll area's viewport, for its
 * `onInitialized`, so a list opens on what is chosen. Only the viewport
 * moves: `scrollIntoView` would also scroll whatever clips it, such as the
 * island's boxes, which can be scrolled though they show no scrollbar.
 * `selector` finds the chosen item: a list box's option by default. A strip
 * that scrolls sideways is centred along its length. */
export function revealChosen(
  viewport: HTMLElement,
  selector = '[aria-selected="true"]',
  orientation: "horizontal" | "vertical" = "vertical",
) {
  const chosen = viewport.querySelector<HTMLElement>(selector);
  if (!chosen) return;
  const box = viewport.getBoundingClientRect();
  const item = chosen.getBoundingClientRect();
  if (orientation === "horizontal") {
    viewport.scrollLeft +=
      item.left - box.left - (viewport.clientWidth - item.width) / 2;
    return;
  }
  viewport.scrollTop += item.top - box.top - (box.height - item.height) / 2;
}

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { SettingsSearchEntry } from "./settings-search-index";

const FOCUSABLE = "input, button, [tabindex='0']";

// The control the row's title names, not its first button: a reset sits
// before its control. Within a group of choices, the one chosen.
const control = (row: HTMLElement) => {
  const titleId = row.querySelector("[id$='-title']")?.id;
  const labelled = titleId
    ? row.querySelector<HTMLElement>(
        `[aria-labelledby~="${CSS.escape(titleId)}"]`,
      )
    : null;
  const scope = labelled ?? row;
  if (scope.matches(FOCUSABLE)) return scope;
  return (
    scope.querySelector<HTMLElement>(
      "input[type=radio]:checked, [role=radio][aria-checked=true]",
    ) ??
    scope.querySelector<HTMLElement>("input:not([type=hidden])") ??
    scope.querySelector<HTMLElement>(FOCUSABLE)
  );
};

const find = (pane: HTMLElement, entry: SettingsSearchEntry) => {
  if (entry.target === "pane") return pane;
  if (entry.target === "group")
    return (
      [...pane.querySelectorAll("h2")]
        .find((heading) => heading.textContent === entry.title)
        ?.closest("section") ?? null
    );
  return (
    [...pane.querySelectorAll<HTMLElement>("[data-control-row]")].find(
      (row) => row.dataset.controlRow === entry.title,
    ) ?? null
  );
};

// Brings an opened result into view in its pane and flashes it, so the eye
// lands on it, then hands its control the keyboard focus. Returns false while
// the pane has not drawn it yet.
export function revealSetting(
  pane: HTMLElement,
  entry: SettingsSearchEntry,
): boolean {
  const target = find(pane, entry);
  if (!target) return false;
  target.scrollIntoView({
    block: entry.target === "pane" ? "start" : "center",
  });
  if (entry.target === "row") {
    const theme = getComputedStyle(target);
    const fill = theme.getPropertyValue("--color-fill-strong");
    const reach = theme.getPropertyValue("--spacing-inset-sm");
    // A shadow as wide as the tight inset carries the fill past the row's
    // edges, so the flash reads as a rounded highlight rather than a band.
    target.animate(
      [
        {
          backgroundColor: fill,
          borderRadius: theme.getPropertyValue("--radius-part"),
          boxShadow: `0 0 0 ${reach} ${fill}`,
        },
        {
          backgroundColor: "transparent",
          boxShadow: `0 0 0 ${reach} transparent`,
        },
      ],
      { duration: 1600, easing: "ease-out" },
    );
    if (target instanceof HTMLElement)
      control(target)?.focus({ preventScroll: true });
  }
  return true;
}

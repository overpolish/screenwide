// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

const textInputTypes = new Set([
  "email",
  "number",
  "password",
  "search",
  "tel",
  "text",
  "url",
]);

export const ownsTextEditingKeys = (target: EventTarget | null) =>
  (target instanceof HTMLInputElement && textInputTypes.has(target.type)) ||
  target instanceof HTMLTextAreaElement ||
  (target instanceof HTMLElement && target.isContentEditable);

// Widgets whose own arrow-key behaviour is the point of them: react-aria
// sliders, number fields, listboxes, menus and tab lists all move a value or a
// highlight with the arrows. Matching the ancestors too catches the composed
// parts (an option inside a listbox, a thumb inside a slider) that receive the
// key event instead of the widget root.
const arrowKeyRoles = [
  "combobox",
  "listbox",
  "menu",
  "option",
  "radiogroup",
  "slider",
  "spinbutton",
  "tab",
  "toolbar",
];
const arrowKeyTargets = [
  "select",
  ...arrowKeyRoles.map((role) => `[role="${role}"]`),
].join(",");

// Groups whose arrows only move between their items, as opposed to changing a
// value: a tool picked from a toolbar with the pointer is not a request to walk
// the toolbar with the keyboard.
const focusOnlyRoles = ["radiogroup", "tab", "tablist", "toolbar"];
const focusOnlyTargets = focusOnlyRoles
  .map((role) => `[role="${role}"]`)
  .join(",");
const valueTargets = [
  "select",
  ...arrowKeyRoles
    .filter((role) => !focusOnlyRoles.includes(role))
    .map((role) => `[role="${role}"]`),
].join(",");

/** Whether the arrows on `target` would only move focus or the choice between
 * a group's items, rather than change a value inside one of them. */
export const arrowsOnlyMoveFocus = (target: EventTarget | null) =>
  target instanceof HTMLElement &&
  target.closest(focusOnlyTargets) !== null &&
  !(target instanceof HTMLInputElement && target.type === "range") &&
  target.closest(valueTargets) === null;

export const ownsArrowKeys = (target: EventTarget | null) =>
  target instanceof HTMLElement &&
  // A range input carries the slider role implicitly, so it needs its own test.
  ((target instanceof HTMLInputElement && target.type === "range") ||
    target.closest(arrowKeyTargets) !== null);

const activationTargets = [
  "a[href]",
  "button",
  "input",
  "select",
  "summary",
  '[role="button"]',
  '[role="checkbox"]',
  '[role="combobox"]',
  '[role="radio"]',
  '[role="slider"]',
  '[role="spinbutton"]',
  '[role="switch"]',
  '[role="tab"]',
].join(",");

export const ownsActivationKeys = (target: EventTarget | null) =>
  target instanceof HTMLElement && target.closest(activationTargets) !== null;

const popupInteractionTargets = [
  '[aria-expanded="true"]',
  '[role="listbox"]',
  '[role="menu"]',
  '[role="menuitem"]',
  '[role="option"]',
].join(",");

export const ownsPopupInteractionKeys = (target: EventTarget | null) =>
  target instanceof HTMLElement &&
  target.closest(popupInteractionTargets) !== null;

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, expect, it, vi } from "vitest";

const interaction = vi.hoisted<{ modality: string | null }>(() => ({
  modality: "pointer",
}));
vi.mock("react-aria/private/interactions/useFocusVisible", () => ({
  getInteractionModality: () => interaction.modality,
  setInteractionModality: (value: string) => {
    interaction.modality = value;
  },
}));

import { installKeyboardNavigationModality } from "./keyboard-navigation-modality";

let dispose: () => void;
beforeEach(() => {
  interaction.modality = "pointer";
  vi.stubGlobal("window", new EventTarget());
  vi.stubGlobal("document", new EventTarget());
  // React Aria observes document capture before this module's restoration.
  for (const type of ["keydown", "keyup"]) {
    document.addEventListener(type, () => {
      interaction.modality = "keyboard";
    });
  }
  dispose = installKeyboardNavigationModality();
});
afterEach(() => {
  dispose();
  vi.unstubAllGlobals();
});

const key = (
  type: string,
  value: string,
  modifiers: Partial<
    Pick<KeyboardEvent, "altKey" | "ctrlKey" | "metaKey">
  > = {},
) => {
  const event = Object.assign(new Event(type, { cancelable: true }), {
    altKey: false,
    ctrlKey: false,
    key: value,
    metaKey: false,
    ...modifiers,
  });
  window.dispatchEvent(event);
  document.dispatchEvent(event);
  return event;
};

it("starts a fresh webview without a focus ring", () => {
  dispose();
  interaction.modality = null;
  dispose = installKeyboardNavigationModality();
  expect(interaction.modality).toBe("pointer");
  key("keydown", "Tab");
  expect(interaction.modality).toBe("keyboard");
});

it.each(["pointer", "keyboard", "virtual"])(
  "preserves existing %s interaction during installation",
  (modality) => {
    dispose();
    interaction.modality = modality;
    dispose = installKeyboardNavigationModality();
    expect(interaction.modality).toBe(modality);
  },
);

it("promotes unmodified navigation on keydown and keyup", () => {
  for (const value of [
    "Tab",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ArrowUp",
    "Home",
    "End",
    "PageUp",
    "PageDown",
  ]) {
    interaction.modality = "pointer";
    expect(key("keydown", value).defaultPrevented).toBe(false);
    expect(interaction.modality).toBe("keyboard");
    interaction.modality = "virtual";
    key("keyup", value);
    expect(interaction.modality).toBe("keyboard");
  }
});

it("restores pointer or virtual modality for ordinary and modified keys", () => {
  for (const initial of ["pointer", "virtual", "keyboard"]) {
    interaction.modality = initial;
    key("keydown", "Enter");
    expect(interaction.modality).toBe(initial);
    key("keyup", "a");
    expect(interaction.modality).toBe(initial);
  }
  for (const modifiers of [
    { ctrlKey: true },
    { metaKey: true },
    { altKey: true },
  ]) {
    interaction.modality = "pointer";
    key("keydown", "ArrowDown", modifiers);
    expect(interaction.modality).toBe("pointer");
  }
});

it("cleans up both capture phases", () => {
  dispose();
  interaction.modality = "pointer";
  key("keydown", "ArrowDown");
  expect(interaction.modality).toBe("keyboard");
});

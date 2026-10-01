// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, expect, it, vi } from "vitest";

const interaction = vi.hoisted(() => ({ modality: "pointer" }));
vi.mock("react-aria/private/interactions/useFocusVisible", () => ({
  getInteractionModality: () => interaction.modality,
  setInteractionModality: (value: string) => {
    interaction.modality = value;
  },
}));

import { preserveEscapeFocus } from "./escape-focus";

let dispose: () => void;
beforeEach(() => {
  interaction.modality = "pointer";
  vi.stubGlobal("window", new EventTarget());
  vi.stubGlobal("document", new EventTarget());
  // React Aria observes both phases at document capture, before our listener.
  for (const type of ["keydown", "keyup"]) {
    document.addEventListener(type, () => {
      interaction.modality = "keyboard";
    });
  }
  dispose = preserveEscapeFocus();
});
afterEach(() => {
  dispose();
  vi.unstubAllGlobals();
});

const key = (type: string, value: string) => {
  const event = Object.assign(new Event(type, { cancelable: true }), {
    key: value,
  });
  window.dispatchEvent(event);
  document.dispatchEvent(event);
  return event;
};

it("keeps unused Escape presses, repeats and release in pointer modality", () => {
  for (const type of ["keydown", "keydown", "keyup"]) {
    expect(key(type, "Escape").defaultPrevented).toBe(false);
    expect(interaction.modality).toBe("pointer");
  }
});

it("leaves Escape available to cancellation handlers without a focus ring", () => {
  const cancel = vi.fn(() => {
    expect(interaction.modality).toBe("pointer");
  });
  document.addEventListener("keydown", cancel);
  key("keydown", "Escape");
  expect(cancel).toHaveBeenCalledOnce();
});

it("preserves existing keyboard focus and lets Tab start keyboard navigation", () => {
  key("keydown", "Tab");
  expect(interaction.modality).toBe("keyboard");
  key("keydown", "Escape");
  key("keyup", "Escape");
  expect(interaction.modality).toBe("keyboard");
});

it("handles an Escape release when native dismissal consumed keydown", () => {
  key("keyup", "Escape");
  expect(interaction.modality).toBe("pointer");
});

it("removes its listeners on cleanup", () => {
  dispose();
  key("keydown", "Escape");
  expect(interaction.modality).toBe("keyboard");
});

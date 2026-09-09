// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const interaction = vi.hoisted(() => ({ modality: "pointer" }));
vi.mock("react-aria/private/interactions/useFocusVisible", () => ({
  getInteractionModality: () => interaction.modality,
  setInteractionModality: (value: string) => {
    interaction.modality = value;
  },
}));

import { installPointerModalityGuard } from "./pointer-modality";

describe("pointer focus restoration", () => {
  let page: EventTarget & { activeElement: object; visibilityState: string };
  let panel: EventTarget;
  const closeButton = {};

  beforeEach(() => {
    vi.useFakeTimers();
    interaction.modality = "pointer";
    page = Object.assign(new EventTarget(), {
      activeElement: closeButton,
      visibilityState: "visible",
    });
    panel = new EventTarget();
    vi.stubGlobal("document", page);
    vi.stubGlobal("window", panel);
    // Model React Aria interpreting an unsolicited element focus as virtual.
    panel.addEventListener("focus", (event) => {
      if (event.target !== panel) interaction.modality = "virtual";
    });
    installPointerModalityGuard();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  function focusElement(element: object) {
    page.activeElement = element;
    const event = new Event("focus");
    Object.defineProperty(event, "target", { value: element });
    panel.dispatchEvent(event);
  }

  it("keeps a mouse-focused close button ring-free after a delayed reopen", () => {
    page.dispatchEvent(new Event("pointerdown"));
    panel.dispatchEvent(new Event("blur"));
    vi.advanceTimersByTime(5000);
    panel.dispatchEvent(new Event("focus"));
    focusElement(closeButton);
    vi.runAllTimers();
    expect(interaction.modality).toBe("pointer");
  });

  it("handles element restoration without a window focus event", () => {
    panel.dispatchEvent(new Event("blur"));
    focusElement(closeButton);
    expect(interaction.modality).toBe("pointer");
  });

  it("handles the captured WebKit double-focus sequence before becoming visible", () => {
    page.dispatchEvent(new Event("pointerdown"));
    page.visibilityState = "hidden";
    panel.dispatchEvent(new Event("blur"));
    page.dispatchEvent(new Event("visibilitychange"));
    vi.advanceTimersByTime(5000);
    focusElement(closeButton);
    panel.dispatchEvent(new Event("focus"));
    vi.runAllTimers();
    focusElement(closeButton);
    expect(interaction.modality).toBe("pointer");
    page.visibilityState = "visible";
    page.dispatchEvent(new Event("visibilitychange"));
    vi.runAllTimers();
    expect(interaction.modality).toBe("pointer");
    // Later unsolicited focus is no longer part of native restoration.
    focusElement(closeButton);
    expect(interaction.modality).toBe("virtual");
  });

  it("lets keyboard input cancel restoration while the webview is hidden", () => {
    page.visibilityState = "hidden";
    panel.dispatchEvent(new Event("blur"));
    focusElement(closeButton);
    page.dispatchEvent(new Event("keydown"));
    interaction.modality = "keyboard";
    page.visibilityState = "visible";
    page.dispatchEvent(new Event("visibilitychange"));
    vi.runAllTimers();
    expect(interaction.modality).toBe("keyboard");
  });

  it("preserves mouse modality when WebKit restores a different control after menu use", () => {
    page.activeElement = {}; // The screenshot-menu trigger before hiding.
    page.dispatchEvent(new Event("pointerdown"));
    page.visibilityState = "hidden";
    panel.dispatchEvent(new Event("blur"));
    page.dispatchEvent(new Event("visibilitychange"));
    vi.advanceTimersByTime(5000);
    focusElement(closeButton);
    panel.dispatchEvent(new Event("focus"));
    vi.runAllTimers();
    focusElement(closeButton);
    expect(interaction.modality).toBe("pointer");
    page.visibilityState = "visible";
    page.dispatchEvent(new Event("visibilitychange"));
    vi.runAllTimers();
    expect(interaction.modality).toBe("pointer");
    focusElement({});
    expect(interaction.modality).toBe("virtual");
  });

  it("lets keyboard input cancel a hidden-to-visible restoration snapshot", () => {
    page.visibilityState = "hidden";
    panel.dispatchEvent(new Event("blur"));
    page.dispatchEvent(new Event("visibilitychange"));
    focusElement({});
    panel.dispatchEvent(new Event("focus"));
    page.dispatchEvent(new Event("keydown"));
    interaction.modality = "keyboard";
    page.visibilityState = "visible";
    page.dispatchEvent(new Event("visibilitychange"));
    vi.runAllTimers();
    expect(interaction.modality).toBe("keyboard");
  });

  it("does not override virtual focus moving to another control", () => {
    panel.dispatchEvent(new Event("blur"));
    panel.dispatchEvent(new Event("focus"));
    focusElement({});
    vi.runAllTimers();
    expect(interaction.modality).toBe("virtual");
  });

  it("does not overwrite keyboard input with a queued pointer restoration", () => {
    panel.dispatchEvent(new Event("blur"));
    panel.dispatchEvent(new Event("focus"));
    page.dispatchEvent(new Event("keydown"));
    interaction.modality = "keyboard";
    vi.runAllTimers();
    expect(interaction.modality).toBe("keyboard");
  });

  it("preserves keyboard modality when a keyboard-focused panel reopens", () => {
    interaction.modality = "keyboard";
    panel.dispatchEvent(new Event("blur"));
    panel.dispatchEvent(new Event("focus"));
    vi.runAllTimers();
    expect(interaction.modality).toBe("keyboard");
  });
});

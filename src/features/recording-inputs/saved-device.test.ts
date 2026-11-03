// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { findSavedDevice } from "./saved-device";

const saved = { id: "usb#port-9#c920", label: "HD Pro Webcam C920" };

describe("findSavedDevice", () => {
  it("finds a device moved to another port by its name", () => {
    const moved = { id: "usb#port-7#c920", label: "HD Pro Webcam C920" };
    expect(
      findSavedDevice(saved, [{ id: "sony", label: "Sony Camera" }, moved]),
    ).toBe(moved);
  });

  it("prefers the same id over another device of the same name", () => {
    const same = { ...saved };
    const twin = { id: "usb#port-7#c920", label: saved.label };
    expect(findSavedDevice(saved, [twin, same])).toBe(same);
  });

  it("chooses neither of two devices sharing the saved name", () => {
    expect(
      findSavedDevice(saved, [
        { id: "usb#port-1#c920", label: saved.label },
        { id: "usb#port-2#c920", label: saved.label },
      ]),
    ).toBeNull();
  });

  it("finds nothing when the device is gone", () => {
    expect(findSavedDevice(saved, [{ id: "sony", label: "Sony" }])).toBeNull();
  });
});

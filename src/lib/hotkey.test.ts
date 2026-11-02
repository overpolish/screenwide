// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  hotkeyFromEvent,
  hotkeyKeys,
  hotkeysEqual,
  hotkeyValue,
  keyCodeFromEvent,
  mouseControlFromButton,
  parseHotkey,
} from "./hotkey";

const key = {
  altKey: false,
  code: "KeyR",
  ctrlKey: false,
  isComposing: false,
  key: "r",
  metaKey: false,
  repeat: false,
  shiftKey: false,
};

describe("hotkey capture", () => {
  it("captures a single physical key, including sided modifiers", () => {
    expect(hotkeyFromEvent(key, "single-control")).toBe("KeyR");
    expect(
      hotkeyFromEvent(
        { ...key, altKey: true, code: "AltRight", key: "Alt" },
        "single-control",
      ),
    ).toBe("AltRight");
    expect(
      hotkeyFromEvent(
        { ...key, code: "Delete", key: "Delete" },
        "single-control",
      ),
    ).toBe("Delete");
    for (const overrides of [
      { repeat: true },
      { isComposing: true },
      { code: "Unidentified" },
      { key: "Dead" },
    ]) {
      expect(
        hotkeyFromEvent({ ...key, ...overrides }, "single-control"),
      ).toBeNull();
    }
  });
  it("only accepts supported auxiliary mouse buttons", () => {
    expect(mouseControlFromButton(0)).toBeNull();
    expect(mouseControlFromButton(2)).toBeNull();
    expect(mouseControlFromButton(1)).toBe("MouseMiddle");
    expect(mouseControlFromButton(3)).toBe("MouseBack");
    expect(mouseControlFromButton(4)).toBe("MouseForward");
  });
  it("formats single controls using the existing Keyboard mapping", () => {
    expect(hotkeyKeys("MetaRight", true)).toEqual(["Command"]);
    expect(hotkeyKeys("MetaLeft", false)).toEqual(["Win"]);
    expect(hotkeyKeys("AltLeft", true)).toEqual(["Option"]);
    expect(hotkeyKeys("ShiftRight", true)).toEqual(["Shift"]);
    expect(hotkeyKeys("MouseBack", false)).toEqual(["MouseBack"]);
  });
  it("requires a modifier and uses physical key codes", () => {
    expect(hotkeyFromEvent(key)).toBeNull();
    expect(hotkeyFromEvent({ ...key, ctrlKey: true, key: "к" })).toBe(
      "Control+KeyR",
    );
    expect(hotkeyFromEvent({ ...key, metaKey: true, shiftKey: true })).toBe(
      "Super+Shift+KeyR",
    );
  });
  it("refuses Shift alone for a shortcut that works anywhere", () => {
    expect(hotkeyFromEvent({ ...key, shiftKey: true })).toBeNull();
    expect(hotkeyFromEvent({ ...key, shiftKey: true }, "local-shortcut")).toBe(
      "Shift+KeyR",
    );
    expect(keyCodeFromEvent({ ...key, shiftKey: true })).toBe("KeyR");
    expect(
      keyCodeFromEvent({ ...key, code: "ShiftLeft", key: "Shift" }),
    ).toBeNull();
  });
  it("reads a shortcut's key and modifiers, aliases resolved", () => {
    expect(parseHotkey("CommandOrControl+Shift+KeyS", true)).toEqual({
      code: "KeyS",
      modifiers: ["Super", "Shift"],
    });
    expect(parseHotkey("CommandOrControl+Shift+KeyS", false)).toEqual({
      code: "KeyS",
      modifiers: ["Control", "Shift"],
    });
    expect(hotkeyValue("KeyS", ["Shift", "Alt", "Super"])).toBe(
      "Super+Alt+Shift+KeyS",
    );
  });
  it("names punctuation by what is printed on it", () => {
    expect(hotkeyKeys("Super+Shift+Slash", true)).toEqual([
      "Shift",
      "Command",
      "/",
    ]);
    expect(hotkeyKeys("Control+Backquote", false)).toEqual(["Control", "`"]);
  });
  it("allows local unmodified keys and preserves the pressed modifier", () => {
    expect(
      hotkeyFromEvent({ ...key, code: "Tab", key: "Tab" }, "local-shortcut"),
    ).toBe("Tab");
    expect(hotkeyFromEvent({ ...key, metaKey: true }, "local-shortcut")).toBe(
      "Super+KeyR",
    );
  });
  it("ignores modifiers, repeats, composition and unidentified keys", () => {
    for (const overrides of [
      { code: "ShiftLeft", key: "Shift" },
      { repeat: true },
      { isComposing: true },
      { code: "Unidentified" },
      { key: "Dead" },
    ])
      expect(
        hotkeyFromEvent({ ...key, ctrlKey: true, ...overrides }),
      ).toBeNull();
  });
  it("formats platform modifiers in the platform's order, key last", () => {
    expect(hotkeyKeys("CommandOrControl+Alt+Digit1", true)).toEqual([
      "Option",
      "Command",
      "1",
    ]);
    expect(hotkeyKeys("CommandOrControl+Alt+Digit1", false)).toEqual([
      "Control",
      "Alt",
      "1",
    ]);
    expect(hotkeyKeys("Super+Control+Alt+Shift+KeyK", true)).toEqual([
      "Control",
      "Option",
      "Shift",
      "Command",
      "K",
    ]);
    expect(hotkeyKeys("Shift+Super+Alt+Control+KeyK", false)).toEqual([
      "Control",
      "Alt",
      "Win",
      "Shift",
      "K",
    ]);
    expect(hotkeyKeys("Super+ArrowUp", false)).toEqual(["Win", "Up"]);
    expect(hotkeyKeys(null, true)).toEqual([]);
  });
  it("recognizes equivalent modifier aliases and order", () => {
    expect(
      hotkeysEqual("Shift+Super+KeyR", "CommandOrControl+Shift+KeyR", true),
    ).toBe(true);
    expect(hotkeysEqual(null, "KeyR")).toBe(false);
  });
});

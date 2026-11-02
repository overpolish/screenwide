// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { assignFromMap } from "./map-assignment";
import { reservedUse } from "./reserved-shortcuts";
import {
  clashes,
  type ConflictSubject,
  conflictOf,
  type ShortcutScope,
} from "./shortcut-conflicts";
import { bestSuggestion, suggestShortcuts } from "./shortcut-suggestions";

const shortcut = (
  id: string,
  value: string | null,
  scope: ShortcutScope = "anywhere",
): ConflictSubject => ({ id, kind: "combo", label: id, scope, value });

// The shortcuts a fresh install has, give or take: Take a screenshot on
// ⌘⇧S, and a moment on ⌘⇧1, which counts as much as any other.
const installed = [
  shortcut("Take a screenshot", "CommandOrControl+Shift+KeyS"),
  shortcut("Place Funny", "CommandOrControl+Shift+Digit1"),
  shortcut("Copy measurement", "CommandOrControl+KeyC", "ruler"),
];

const conflictsFor =
  (isMac: boolean, subject = shortcut("new", null)) =>
  (value: string) =>
    conflictOf({ isMac, shortcuts: installed, subject, value });

const offered = (code: string, isMac: boolean) =>
  suggestShortcuts(code, { conflictOf: conflictsFor(isMac), isMac });

describe("shortcut suggestions", () => {
  it("ranks a Mac's combinations ⌘⇧, ⌃⌥, ⌘⌥, ⌃⇧", () => {
    expect(offered("KeyK", true).map(({ value }) => value)).toEqual([
      "Super+Shift+KeyK",
      "Control+Alt+KeyK",
      "Super+Alt+KeyK",
      "Control+Shift+KeyK",
    ]);
  });

  it("ranks Windows' Ctrl+Shift first and never offers the Windows key", () => {
    const values = offered("KeyK", false).map(({ value }) => value);
    expect(values[0]).toBe("Control+Shift+KeyK");
    expect(values[values.length - 1]).toBe("Control+Alt+KeyK");
    expect(values.some((value) => value.includes("Super"))).toBe(false);
  });

  it("marks a taken combination and never offers it first", () => {
    const suggestions = offered("KeyS", true);
    expect(suggestions[0]).toMatchObject({
      conflict: { kind: "taken", label: "Take a screenshot" },
      value: "Super+Shift+KeyS",
    });
    expect(bestSuggestion(suggestions)?.value).toBe("Control+Alt+KeyS");
  });

  it("counts a moment's key as taken", () => {
    expect(bestSuggestion(offered("Digit1", true))?.value).toBe(
      "Control+Alt+Digit1",
    );
    expect(
      conflictOf({
        isMac: true,
        shortcuts: installed,
        subject: shortcut("Start or stop recording", null),
        value: "Super+Shift+Digit1",
      }),
    ).toEqual({ kind: "taken", label: "Place Funny" });
  });

  it("adds a free one when every one shown clashes", () => {
    const busy = (value: string) =>
      value.endsWith("KeyJ") && !value.startsWith("Super+Control+")
        ? { kind: "taken" as const, label: "Busy" }
        : null;
    const suggestions = suggestShortcuts("KeyJ", {
      conflictOf: busy,
      isMac: true,
    });
    expect(suggestions).toHaveLength(5);
    expect(bestSuggestion(suggestions)?.value).toBe("Super+Control+KeyJ");
  });

  it("keeps a chord held down in view, marked, but not first", () => {
    const suggestions = suggestShortcuts("KeyC", {
      conflictOf: conflictsFor(true),
      include: [["Super"]],
      isMac: true,
    });
    expect(suggestions[suggestions.length - 1]).toMatchObject({
      conflict: { by: "apps", kind: "reserved" },
      value: "Super+KeyC",
    });
    expect(bestSuggestion(suggestions)?.value).toBe("Super+Shift+KeyC");
  });

  it("leaves out a held combination of Shift alone", () => {
    const suggestions = suggestShortcuts("KeyC", {
      conflictOf: conflictsFor(true),
      include: [["Shift"], []],
      isMac: true,
    });
    expect(suggestions).toHaveLength(4);
  });
});

describe("shortcut clashes", () => {
  it("only meets shortcuts of its own scope", () => {
    expect(
      conflictOf({
        isMac: true,
        shortcuts: installed,
        subject: shortcut("Copy text", null, "ocr"),
        value: "Super+KeyC",
      }),
    ).toBeNull();
    expect(
      conflictOf({
        isMac: true,
        shortcuts: installed,
        subject: shortcut("Undo", null, "ruler"),
        value: "Super+KeyC",
      }),
    ).toEqual({ kind: "taken", label: "Copy measurement" });
  });

  it("reserves the system's and apps' combinations for shortcuts that work anywhere", () => {
    expect(reservedUse("Super+Space", true)?.by).toBe("system");
    expect(reservedUse("CommandOrControl+KeyV", true)?.by).toBe("apps");
    expect(reservedUse("Control+KeyV", false)?.by).toBe("apps");
    expect(reservedUse("Control+Alt+KeyV", true)).toBeNull();
  });

  it("names what each platform's combinations are used for", () => {
    const purpose = (value: string, isMac: boolean) =>
      reservedUse(value, isMac)?.label();
    expect(purpose("Super+KeyZ", true)).toBe("Undo");
    expect(purpose("Super+Shift+KeyZ", true)).toBe("Redo");
    expect(purpose("Super+Shift+Digit3", true)).toBe("screenshots");
    expect(purpose("Super+Control+Shift+Digit4", true)).toBe("screenshots");
    expect(purpose("Control+KeyZ", false)).toBe("Undo");
    expect(purpose("Control+Shift+KeyZ", false)).toBe("Redo");
    expect(purpose("Alt+F4", false)).toBe("closing windows");
  });

  it("reserves every Windows key combination on Windows", () => {
    expect(reservedUse("Super+Shift+KeyK", false)?.by).toBe("system");
    expect(reservedUse("Super+Shift+KeyK", true)).toBeNull();
  });

  it("finds shortcuts that share a combination", () => {
    const found = clashes(
      [
        shortcut("a", "CommandOrControl+Shift+KeyA"),
        shortcut("b", "Super+Shift+KeyA"),
        shortcut("c", "Super+Shift+KeyA", "ruler"),
      ],
      true,
    );
    expect([...found.keys()]).toEqual(["a", "b"]);
  });
});

describe("keys clicked on the keyboard map", () => {
  const assign = ({
    code,
    kind = "combo",
    latched,
    scope = "anywhere",
  }: {
    code: string;
    latched: Parameters<typeof assignFromMap>[0]["latched"];
    kind?: "combo" | "hold";
    scope?: ShortcutScope;
  }) =>
    assignFromMap({
      code,
      conflictOf: conflictsFor(true, shortcut("new", null, scope)),
      isMac: true,
      latched,
      shortcut: { kind, scope },
    });

  it("takes the key with the latched modifiers", () => {
    expect(assign({ code: "KeyK", latched: ["Control", "Alt"] })).toEqual({
      kind: "assign",
      modifiers: ["Control", "Alt"],
      value: "Control+Alt+KeyK",
    });
  });

  it("takes the best free combination with nothing latched, or Shift alone", () => {
    expect(assign({ code: "KeyS", latched: [] })).toMatchObject({
      value: "Control+Alt+KeyS",
    });
    expect(assign({ code: "KeyS", latched: ["Shift"] })).toMatchObject({
      modifiers: ["Control", "Alt"],
      value: "Control+Alt+KeyS",
    });
  });

  it("refuses a taken or reserved combination", () => {
    expect(assign({ code: "KeyS", latched: ["Super", "Shift"] })).toMatchObject(
      {
        conflict: { kind: "taken" },
        kind: "refuse",
      },
    );
    expect(assign({ code: "KeyQ", latched: ["Super"] })).toMatchObject({
      conflict: { kind: "reserved" },
      kind: "refuse",
    });
  });

  it("gives a tool's own shortcut a bare key", () => {
    expect(assign({ code: "KeyK", latched: [], scope: "ruler" })).toMatchObject(
      { value: "KeyK" },
    );
    expect(
      assign({ code: "KeyC", latched: ["Super"], scope: "ruler" }),
    ).toMatchObject({
      kind: "refuse",
    });
  });

  it("gives a held key the key alone", () => {
    expect(
      assign({ code: "AltLeft", kind: "hold", latched: ["Super"] }),
    ).toEqual({
      kind: "assign",
      modifiers: [],
      value: "AltLeft",
    });
  });
});

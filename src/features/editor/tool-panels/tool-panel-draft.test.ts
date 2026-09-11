// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { DEFAULT_CURSOR_EFFECTS } from "../recording-export-settings";

import { resolveToolPanelSnapshot, ToolPanelDraft } from "./tool-panel-draft";
import { ToolPanelSnapshot } from "./tool-panel-store";

const snapshot = (
  sizePercent: number,
  acknowledgedSeq = 0,
): ToolPanelSnapshot => ({
  acknowledgedSeq,
  cursorEffects: { ...DEFAULT_CURSOR_EFFECTS, sizePercent },
  hasCursorData: true,
  isSaving: false,
});
const draft = (sizePercent: number, seq: number): ToolPanelDraft => ({
  seq,
  values: { cursorEffects: { ...DEFAULT_CURSOR_EFFECTS, sizePercent } },
  workspace: "recording",
});
const resolve = (value: ToolPanelSnapshot, pending: ToolPanelDraft | null) =>
  resolveToolPanelSnapshot(value, pending, "recording");

describe("panel input during delayed editor replies", () => {
  it("shows the newest input immediately and ignores earlier drag replies", () => {
    const latest = draft(250, 3);
    for (const reply of [snapshot(100), snapshot(150, 1), snapshot(200, 2)]) {
      expect(resolve(reply, latest).cursorEffects.sizePercent).toBe(250);
    }
  });

  it("waits for the latest sequence even when an older request had the same value", () => {
    const latest = draft(150, 3);
    expect(resolve(snapshot(150, 1), latest).cursorEffects.sizePercent).toBe(
      150,
    );
    expect(resolve(snapshot(200, 2), latest).cursorEffects.sizePercent).toBe(
      150,
    );
  });

  it("accepts the canonical editor value when the latest request is acknowledged", () => {
    const reply = snapshot(245, 3);
    expect(resolve(reply, draft(250, 3))).toBe(reply);
  });

  it("follows subsequent editor changes after acknowledgement", () => {
    const latest = draft(250, 3);
    expect(resolve(snapshot(250, 3), latest).cursorEffects.sizePercent).toBe(
      250,
    );
    expect(resolve(snapshot(100, 3), null).cursorEffects.sizePercent).toBe(100);
  });

  it("keeps saving and availability metadata live while an edit is pending", () => {
    const reply = { ...snapshot(100, 1), hasCursorData: false, isSaving: true };
    const value = resolve(reply, draft(250, 3));
    expect(value.isSaving).toBe(true);
    expect(value.hasCursorData).toBe(false);
    expect(value.cursorEffects.sizePercent).toBe(250);
  });

  it("does not carry another workspace's pending edits across", () => {
    const reply = snapshot(100);
    expect(resolveToolPanelSnapshot(reply, draft(250, 3), "screenshot")).toBe(
      reply,
    );
  });
});

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { DEFAULT_CURSOR_EFFECTS } from "../recording-export-settings";

import { resolveToolPanelSnapshot, ToolPanelDraft } from "./tool-panel-draft";
import {
  DEFAULT_TOOL_PANEL_SNAPSHOT,
  ToolPanelSnapshot,
} from "./tool-panel-store";

const snapshot = (
  sizePercent: number,
  acknowledgedSeq = 0,
): ToolPanelSnapshot => ({
  ...DEFAULT_TOOL_PANEL_SNAPSHOT,
  acknowledgedSeq,
  cursorEffects: { ...DEFAULT_CURSOR_EFFECTS, sizePercent },
  frame: null,
  hasCursorData: true,
  isLocked: false,
  selection: null,
});
const draft = (sizePercent: number, seq: number): ToolPanelDraft => ({
  seq,
  values: { cursorEffects: { ...DEFAULT_CURSOR_EFFECTS, sizePercent } },
  workspace: "recording",
});
const resolve = (value: ToolPanelSnapshot, pending: ToolPanelDraft | null) =>
  resolveToolPanelSnapshot(value, pending, "recording");

const placed = (
  placement: { height: number; width: number; x: number; y: number },
  acknowledgedSeq = 0,
): ToolPanelSnapshot => ({
  ...snapshot(100, acknowledgedSeq),
  selection: {
    ...placement,
    dropShadow: true,
    inset: 0,
    insetMaximum: 2338,
    kind: "primary",
    label: "Screen",
    radius: 0,
    sourceHeight: 2338,
    sourceWidth: 3600,
  },
});

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

  it("keeps lock and availability metadata live while an edit is pending", () => {
    const reply = { ...snapshot(100, 1), hasCursorData: false, isLocked: true };
    const value = resolve(reply, draft(250, 3));
    expect(value.isLocked).toBe(true);
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

describe("selection placement while a field is open", () => {
  const live = { height: 2338, width: 3600, x: 0, y: 0 };

  it("holds only the field being edited over a live drag", () => {
    const editing: ToolPanelDraft = {
      seq: 4,
      values: { selectionOutput: { width: 1800 } },
      workspace: "recording",
    };
    const dragged = placed({ height: 1200, width: 1849, x: 40, y: 12 });

    expect(resolve(dragged, editing).selection).toMatchObject({
      height: 1200,
      width: 1800,
      x: 40,
      y: 12,
    });
  });

  it("lets the editor's answer through once it is acknowledged", () => {
    const editing: ToolPanelDraft = {
      seq: 4,
      values: { selectionOutput: { width: 1800 } },
      workspace: "recording",
    };

    expect(resolve(placed(live, 4), editing).selection).toMatchObject(live);
  });

  it("never shows a reset as a local value", () => {
    const reset: ToolPanelDraft = {
      seq: 5,
      values: { resetSelection: true },
      workspace: "recording",
    };
    const resolved = resolve(placed(live), reset);

    expect(resolved.selection).toMatchObject(live);
    expect(resolved).not.toHaveProperty("resetSelection");
  });
});

describe("canvas size while a field is open", () => {
  const sized = (
    frame: { height: number; width: number },
    acknowledgedSeq = 0,
  ): ToolPanelSnapshot => ({
    ...snapshot(100, acknowledgedSeq),
    frame: { ...frame, sourceHeight: 2338, sourceWidth: 3600 },
  });
  const live = { height: 2338, width: 3600 };

  it("holds only the field being edited over a live frame drag", () => {
    const editing: ToolPanelDraft = {
      seq: 4,
      values: { frameSize: { width: 1800 } },
      workspace: "recording",
    };

    expect(
      resolve(sized({ height: 1200, width: 1849 }), editing).frame,
    ).toEqual({
      height: 1200,
      sourceHeight: 2338,
      sourceWidth: 3600,
      width: 1800,
    });
  });

  it("lets the editor's answer through once it is acknowledged", () => {
    const editing: ToolPanelDraft = {
      seq: 4,
      values: { frameSize: { width: 1800 } },
      workspace: "recording",
    };

    expect(resolve(sized(live, 4), editing).frame).toMatchObject(live);
  });

  it("never shows a frame reset as a local value", () => {
    const reset: ToolPanelDraft = {
      seq: 5,
      values: { resetFrame: true },
      workspace: "recording",
    };
    const resolved = resolve(sized(live), reset);

    expect(resolved.frame).toMatchObject(live);
    expect(resolved).not.toHaveProperty("resetFrame");
  });

  it("leaves a selection edit alone while a size is pending", () => {
    const editing: ToolPanelDraft = {
      seq: 6,
      values: { frameSize: { height: 1000 } },
      workspace: "recording",
    };

    expect(
      resolve(placed({ height: 2338, width: 3600, x: 0, y: 0 }), editing),
    ).toMatchObject({ frame: null, selection: { width: 3600 } });
  });
  it("keeps the bake switch flipped until the editor answers", () => {
    const flipping: ToolPanelDraft = {
      seq: 7,
      values: { bakeCamera: true },
      workspace: "recording",
    };
    const camera: ToolPanelSnapshot = {
      ...snapshot(100),
      selection: {
        canBake: true,
        dropShadow: true,
        height: 506,
        inset: 0,
        insetMaximum: 0,
        isBaked: false,
        kind: "camera",
        label: "Camera",
        radius: 8,
        sourceHeight: 720,
        sourceWidth: 1280,
        width: 900,
        x: 2592,
        y: 73,
      },
    };

    expect(resolve(camera, flipping).selection).toMatchObject({
      isBaked: true,
    });
    // Once the editor has answered, the switch shows what it committed.
    expect(
      resolve({ ...camera, acknowledgedSeq: 7 }, flipping).selection,
    ).toMatchObject({ isBaked: false });
  });

  it("keeps a dragged volume until the editor answers", () => {
    const dragging: ToolPanelDraft = {
      seq: 8,
      values: { audioVolume: -9 },
      workspace: "recording",
    };
    const audio: ToolPanelSnapshot = {
      ...snapshot(100),
      selection: { decibels: 0, kind: "audio", label: "Microphone" },
    };

    expect(resolve(audio, dragging).selection).toMatchObject({ decibels: -9 });
    // Once acknowledged, the knob shows the level the editor committed.
    expect(
      resolve({ ...audio, acknowledgedSeq: 8 }, dragging).selection,
    ).toMatchObject({ decibels: 0 });
  });
});

describe("a counter's aim while the editor answers", () => {
  const aimed = (angle: number, acknowledgedSeq = 0): ToolPanelSnapshot => ({
    ...snapshot(100, acknowledgedSeq),
    annotation: {
      angle,
      animated: true,
      id: "counter-1",
      kind: "counter",
      style: { align: "left", color: "#ffcc00", head: "none", width: 56 },
    },
  });
  const turning: ToolPanelDraft = {
    seq: 5,
    values: { annotationAngle: -Math.PI / 2 },
    workspace: "recording",
  };

  it("holds the aim just set until the editor answers with it", () => {
    expect(resolve(aimed(0), turning).annotation?.angle).toBe(-Math.PI / 2);
  });

  it("takes the editor's own aim once the request is acknowledged", () => {
    expect(resolve(aimed(Math.PI, 5), turning).annotation?.angle).toBe(Math.PI);
  });
});

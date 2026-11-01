// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { DEFAULT_CURSOR_EFFECTS } from "../export/recording-export-settings";

import { resolveToolPanelSnapshot, ToolPanelDraft } from "./tool-panel-draft";
import {
  DEFAULT_TOOL_PANEL_SNAPSHOT,
  ToolPanelMicrophone,
  ToolPanelSnapshot,
} from "./tool-panel-store";

import type { OverlaySize } from "../../../bindings/OverlaySize";

const snapshot = (
  size: OverlaySize,
  acknowledgedSeq = 0,
): ToolPanelSnapshot => ({
  ...DEFAULT_TOOL_PANEL_SNAPSHOT,
  acknowledgedSeq,
  cursorEffects: { ...DEFAULT_CURSOR_EFFECTS, size },
  frame: null,
  hasCursorData: true,
  isLocked: false,
  selection: null,
});
const draft = (size: OverlaySize, seq: number): ToolPanelDraft => ({
  seq,
  values: { cursorEffects: { ...DEFAULT_CURSOR_EFFECTS, size } },
  workspace: "recording",
});
const resolve = (value: ToolPanelSnapshot, pending: ToolPanelDraft | null) =>
  resolveToolPanelSnapshot(value, pending, "recording");

const layered = (acknowledgedSeq = 0): ToolPanelSnapshot => ({
  ...snapshot("standard", acknowledgedSeq),
  selection: {
    dropShadow: true,
    inset: 0,
    kind: "primary",
    label: "Screen",
    radius: 0,
  },
});

describe("panel input during delayed editor replies", () => {
  it("shows the newest input immediately and ignores earlier drag replies", () => {
    const latest = draft("giant", 3);
    for (const reply of [
      snapshot("standard"),
      snapshot("large", 1),
      snapshot("extra-large", 2),
    ]) {
      expect(resolve(reply, latest).cursorEffects.size).toBe("giant");
    }
  });

  it("waits for the latest sequence even when an older request had the same value", () => {
    const latest = draft("large", 3);
    expect(resolve(snapshot("large", 1), latest).cursorEffects.size).toBe(
      "large",
    );
    expect(resolve(snapshot("extra-large", 2), latest).cursorEffects.size).toBe(
      "large",
    );
  });

  it("accepts the canonical editor value when the latest request is acknowledged", () => {
    const reply = snapshot("large", 3);
    expect(resolve(reply, draft("extra-large", 3))).toBe(reply);
  });

  it("follows subsequent editor changes after acknowledgement", () => {
    const latest = draft("giant", 3);
    expect(resolve(snapshot("giant", 3), latest).cursorEffects.size).toBe(
      "giant",
    );
    expect(resolve(snapshot("standard", 3), null).cursorEffects.size).toBe(
      "standard",
    );
  });

  it("keeps lock and availability metadata live while an edit is pending", () => {
    const reply = {
      ...snapshot("standard", 1),
      hasCursorData: false,
      isLocked: true,
    };
    const value = resolve(reply, draft("giant", 3));
    expect(value.isLocked).toBe(true);
    expect(value.hasCursorData).toBe(false);
    expect(value.cursorEffects.size).toBe("giant");
  });

  it("does not carry another workspace's pending edits across", () => {
    const reply = snapshot("standard");
    expect(
      resolveToolPanelSnapshot(reply, draft("giant", 3), "screenshot"),
    ).toBe(reply);
  });
});

describe("actions while the editor answers", () => {
  it("never shows an action as a local value", () => {
    const acting: ToolPanelDraft = {
      seq: 5,
      values: {
        cutStripItem: true,
        deleteChosenShortcuts: true,
        lullPlay: "squeeze",
        reverseAnnotation: true,
        shuffleAnnotation: true,
        trackEnabled: { isEnabled: false, trackId: "camera" },
        trackVolume: { decibels: -6, trackId: "audio:0" },
        trimLullToBreath: true,
      },
      workspace: "recording",
    };
    const resolved = resolve(layered(), acting);

    for (const key of Object.keys(acting.values))
      expect(resolved).not.toHaveProperty(key);
  });
});

describe("the Sound island's switches while the editor answers", () => {
  const microphone: ToolPanelMicrophone = {
    autoVolume: "on",
    autoVolumeProgress: 0,
    ducking: null,
    noise: "off",
    noiseProgress: 0,
    silences: { count: 0, durationMs: 0, progress: null, status: "idle" },
    studio: "off",
    studioModel: null,
    studioProgress: 0,
    voice: "on",
    voiceProgress: 0,
  };
  const sounding: ToolPanelSnapshot = {
    ...snapshot("standard"),
    soundTracks: [
      { decibels: 0, id: "audio:0", label: "Microphone", microphone },
      { decibels: 0, id: "audio:1", label: "System audio" },
    ],
  };

  it("flips the microphone's switches at once, with nothing selected", () => {
    const flipping: ToolPanelDraft = {
      seq: 1,
      values: { autoVolume: false, reduceNoise: true },
      workspace: "recording",
    };
    const [voice, system] = resolve(sounding, flipping).soundTracks ?? [];

    expect(voice.microphone).toMatchObject({
      autoVolume: "off",
      noise: "cleaning",
      voice: "on",
    });
    expect(system).toEqual(sounding.soundTracks?.[1]);
  });
});

describe("canvas size while a field is open", () => {
  const sized = (
    frame: { height: number; width: number },
    acknowledgedSeq = 0,
  ): ToolPanelSnapshot => ({
    ...snapshot("standard", acknowledgedSeq),
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

    const reply = layered();
    expect(resolve(reply, editing)).toMatchObject({
      frame: null,
      selection: reply.selection,
    });
  });
});

describe("a counter's aim while the editor answers", () => {
  const aimed = (angle: number, acknowledgedSeq = 0): ToolPanelSnapshot => ({
    ...snapshot("standard", acknowledgedSeq),
    annotation: {
      angle,
      animated: true,
      id: "counter-1",
      kind: "counter",
      style: {
        align: "left",
        blur: false,
        color: "#ffcc00",
        handDrawn: false,
        head: "none",
        manual: false,
        radius: 0,
        redaction: "erase",
        shadow: false,
        softness: 0,
        strength: 0,
        tint: false,
        width: 56,
      },
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

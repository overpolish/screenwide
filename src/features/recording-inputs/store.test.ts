// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { beforeEach, describe, expect, it, vi } from "vitest";

import { useRecordingInputStore } from "./store";

import type { GeneralSettings } from "../../bindings/GeneralSettings";

// The inputs' storage as an earlier build left it, read by the store the way
// every window reads it: the frame rate and anti-flicker it held are handed
// to the settings the first time, and never again.

const { calls, stored } = vi.hoisted(() => {
  // Before the store is made: it takes its storage when it is created.
  const stored = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => stored.get(key) ?? null,
    removeItem: (key: string) => stored.delete(key),
    setItem: (key: string, value: string) => stored.set(key, value),
  });
  return {
    calls: { reads: 0, saved: [] as Partial<GeneralSettings>[] },
    stored,
  };
});

vi.mock("../settings/api", () => ({
  getGeneralSettings: () => {
    calls.reads += 1;
    return Promise.resolve({ cameraAntiFlicker: false, recordingFps: 60 });
  },
  setGeneralSettings: (settings: Partial<GeneralSettings>) => {
    calls.saved.push(settings);
    return Promise.resolve(settings);
  },
}));

const STORE = "screenwide-recording-inputs";

const storeEarlierBuild = (state: object) => {
  stored.set(STORE, JSON.stringify({ state, version: 0 }));
};

describe("recording inputs storage", () => {
  beforeEach(() => {
    calls.reads = 0;
    calls.saved.length = 0;
    stored.clear();
  });

  it("hands an earlier build's frame rate and anti-flicker to the settings once", async () => {
    storeEarlierBuild({
      cameraPalById: { "camera-1": false, "camera-2": true },
      fps: 30,
      inputs: { camera: true, microphone: false, systemAudio: false },
    });

    await useRecordingInputStore.persist.rehydrate();
    await vi.waitFor(() => {
      expect(calls.saved).toEqual([
        { cameraAntiFlicker: true, recordingFps: 30 },
      ]);
    });
    expect(useRecordingInputStore.getState().inputs.camera).toBe(true);
    expect(stored.get(STORE)).not.toMatch(/fps|cameraPal/);

    await useRecordingInputStore.persist.rehydrate();
    expect(calls.reads).toBe(1);
  });

  it("leaves the settings alone when an earlier build chose neither", async () => {
    storeEarlierBuild({ inputs: { camera: false } });

    await useRecordingInputStore.persist.rehydrate();
    expect(calls.reads).toBe(0);
  });
});

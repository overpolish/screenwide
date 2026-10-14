// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { beforeEach, describe, expect, it, vi } from "vitest";

import { createWorkspaceMirror } from "./workspace-mirror";

const entries = new Map<string, string>();
vi.stubGlobal("localStorage", {
  getItem: (key: string) => entries.get(key) ?? null,
  setItem: (key: string, value: string) => {
    entries.set(key, value);
  },
});

describe("the workspace mirror", () => {
  beforeEach(() => {
    entries.clear();
  });

  it("never takes up a snapshot an earlier build left in storage", () => {
    entries.set(
      "mirror-recording",
      JSON.stringify({ build: "earlier", snapshot: { style: "old" } }),
    );
    // Stored before snapshots carried their build at all.
    entries.set("mirror-screenshot", JSON.stringify({ style: "old" }));
    const { synchronize, useMirror } = createWorkspaceMirror<{
      style: string;
    }>("mirror");
    expect(useMirror.getState().snapshots).toEqual({});

    // Published by this build, it reaches a window that loads after.
    useMirror.getState().publish("recording", { style: "new" });
    const later = createWorkspaceMirror<{ style: string }>("mirror");
    expect(later.useMirror.getState().snapshots.recording).toEqual({
      style: "new",
    });

    // And a stale write from another window is not taken up either.
    synchronize({
      key: "mirror-recording",
      newValue: entries.get("mirror-screenshot") ?? null,
    } as StorageEvent);
    expect(useMirror.getState().snapshots.recording).toBeUndefined();
  });
});

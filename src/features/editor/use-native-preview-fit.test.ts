// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { afterPreviewLayout } from "./use-native-preview-fit";

const deferred = () => {
  let resolve: () => void = () => undefined;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
};

describe("native fit ordering", () => {
  it("waits for the latest coalesced resize layout before fitting", async () => {
    const first = deferred();
    const latest = deferred();
    const layout = { current: first.promise };
    const fit = vi.fn().mockResolvedValue(undefined);
    const fitting = afterPreviewLayout(layout, fit);
    expect(fit).not.toHaveBeenCalled();
    layout.current = latest.promise;
    first.resolve();
    await first.promise;
    expect(fit).not.toHaveBeenCalled();
    latest.resolve();
    await fitting;
    expect(fit).toHaveBeenCalledOnce();
  });

  it("fits immediately after an already settled layout", async () => {
    const fit = vi.fn().mockResolvedValue(undefined);
    await afterPreviewLayout({ current: Promise.resolve() }, fit);
    expect(fit).toHaveBeenCalledOnce();
  });
});

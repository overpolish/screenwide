// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { PreviewProbeLayer } from "./ruler-preview-probes";

describe("PreviewProbeLayer", () => {
  it("labels an extended probe in world pixels at non-unit zoom", () => {
    const markup = renderToStaticMarkup(
      <PreviewProbeLayer
        probes={[{ axis: "x", end: 30, position: 8, start: 10 }]}
        showLabels
        toScreen={({ x, y }) => ({ x: x * 2, y: y * 2 })}
      />,
    );

    expect(markup).toContain(">20 px</text>");
    expect(markup).not.toContain(">40 px</text>");
  });
});

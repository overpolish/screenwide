// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { RulerLabelLayer } from "./ruler-label-layer";
import { LabelHandles } from "./use-label-handles";

const handles: LabelHandles = {
  beginDrag: () => undefined,
  contextMenu: () => undefined,
  drag: () => undefined,
  endDrag: () => undefined,
  enter: () => undefined,
  isVisible: (key) => key !== "m1",
  leave: () => undefined,
  offset: () => ({ x: 0, y: 0 }),
};

describe("RulerLabelLayer", () => {
  it("renders visible labels without duplicating artifact strokes", () => {
    const markup = renderToStaticMarkup(
      <RulerLabelLayer
        guides={[]}
        handles={handles}
        measurements={[{ height: 20, id: 1, width: 20, x: 5, y: 5 }]}
        probes={[{ axis: "x", end: 20, id: 2, position: 10, start: 10 }]}
        radii={[]}
        style={{}}
        viewport={{ height: 100, width: 100 }}
      />,
    );

    expect(markup).toContain(">10 px</text>");
    expect(markup).not.toContain("20 × 20 px");
    expect(markup).not.toContain("<line");
  });
});

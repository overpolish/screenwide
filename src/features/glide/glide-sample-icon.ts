// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Feather } from "lucide-react";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

// Stands in for an extracted app icon, so Storybook needs no files: a Lucide
// glyph on a rounded, shaded square, the way an app's logo is drawn. Its
// colours are the imagined app's own, not the interface's.

const glyph = renderToStaticMarkup(
  createElement(Feather, {
    color: "white",
    height: 9,
    strokeWidth: 2.25,
    width: 9,
    x: 3.5,
    y: 3.5,
  }),
);

export const sampleAppIcon =
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">' +
      '<defs><linearGradient id="face" x1="0" y1="0" x2="0" y2="1">' +
      '<stop offset="0" stop-color="#7c8cff"/>' +
      '<stop offset="1" stop-color="#4b3fd6"/>' +
      "</linearGradient></defs>" +
      '<rect width="16" height="16" rx="3.6" fill="url(#face)"/>' +
      glyph +
      "</svg>",
  );

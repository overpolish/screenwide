// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RectangleHorizontal, RectangleVertical, Square } from "lucide-react";

/** Whether a canvas is wide, tall or square, drawn as its shape. */
export function ShapeGlyph({
  className,
  size,
}: {
  size: { height: number; width: number };
  className?: string;
}) {
  if (size.width > size.height)
    return <RectangleHorizontal aria-hidden className={className} />;
  if (size.height > size.width)
    return <RectangleVertical aria-hidden className={className} />;
  return <Square aria-hidden className={className} />;
}

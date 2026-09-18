// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

/**
 * A drawing tool follows the annotation that was chosen with it.
 *
 * Both drawing tools hit-test every annotation, so the arrow tool can pick up a
 * counter and the counter tool an arrow. What it must not leave behind is a
 * mismatch: the panel dressing a counter while the next press on empty picture
 * draws an arrow. So choosing an annotation takes up its own tool, and the tool
 * in hand is always the shape the next press makes.
 *
 * The select tool is left alone: picking things up is what it is for, and it
 * draws nothing to disagree with.
 */
export function useToolFollowsAnnotation(
  kind: "arrow" | "counter" | null,
  tool: string | null,
  setTool: (tool: "arrow" | "counter") => void,
) {
  // The workspaces rebuild their setter every render; only the chosen
  // annotation and the tool in hand are worth reacting to.
  const setToolRef = useRef(setTool);
  setToolRef.current = setTool;
  useEffect(() => {
    if (kind === null || (tool !== "arrow" && tool !== "counter")) return;
    if (tool === kind) return;
    setToolRef.current(kind);
  }, [kind, tool]);
}

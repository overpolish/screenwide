// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { followedTool } from "./editor-choice";

import type { AnnotationKind } from "../../annotations/types";

/**
 * Takes up the pen the choice calls for once the gesture that made it has
 * ended (`followedTool`): as the press is let go, as ⌘ stops holding Select,
 * or at once for a choice made off the picture, on the strip.
 */
export function useToolFollowsChoice({
  isHeld,
  isPicturePressed,
  kind,
  setTool,
  tool,
}: {
  isHeld: boolean;
  isPicturePressed: boolean;
  kind: AnnotationKind | null;
  setTool: (tool: AnnotationKind) => void;
  tool: string;
}) {
  // The workspaces rebuild their setter every render; only the choice, the
  // tool and what the gesture is waiting on are worth reacting to.
  const setToolRef = useRef(setTool);
  setToolRef.current = setTool;
  useEffect(() => {
    const next = followedTool({ isHeld, isPicturePressed, kind, tool });
    if (next !== null) setToolRef.current(next);
  }, [isHeld, isPicturePressed, kind, tool]);
}

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import type { FaceTrackingProgress } from "../../../../bindings/FaceTrackingProgress";

/**
 * Whether the face is being found in the camera of the recording open in
 * the editor, which the first scene to follow it starts; told by
 * `src-tauri/src/editor/face_tracking.rs`. One recording is open at a time,
 * so whichever one the editor is told about is this one.
 */
export function useIsFindingFace() {
  const [isFinding, setIsFinding] = useState(false);
  useEffect(() => {
    let stop: (() => void) | undefined;
    let current = true;
    listen<FaceTrackingProgress>("editor://face-tracking-progress", (event) => {
      setIsFinding(event.payload.fraction < 1);
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow finding the face", cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, []);
  return isFinding;
}

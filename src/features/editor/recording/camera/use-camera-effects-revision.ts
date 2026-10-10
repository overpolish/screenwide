// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { listenToCameraEffectsChanged } from "./camera-effects-api";

/**
 * A count that goes up each time the camera of recording `artifactId` is
 * drawn differently, for pictures made of it to be made again.
 */
export function useCameraEffectsRevision(artifactId: number) {
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    let stop: (() => void) | undefined;
    let current = true;
    listenToCameraEffectsChanged((id) => {
      if (id === artifactId) setRevision((previous) => previous + 1);
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the camera's effects", cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, [artifactId]);
  return revision;
}

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useLayoutEffect, useRef } from "react";

/** A crop asked for on the native picture, as `preview://crop-request`
 * reports it: the layer double-clicked to crop it, or none where a press
 * past the picture being cropped puts the crop away. */
type CropRequest = { layerId: number | null };

const CROP_REQUEST_EVENT = "preview://crop-request";

/** Hands each crop asked for on the picture to `onRequest`. */
export function useNativeCropRequest(
  onRequest: (layerId: number | null) => void,
) {
  const requestRef = useRef(onRequest);
  useLayoutEffect(() => {
    requestRef.current = onRequest;
  });
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWindow()
      .listen<CropRequest>(CROP_REQUEST_EVENT, ({ payload }) => {
        requestRef.current(payload.layerId);
      })
      .then((dispose) => {
        if (disposed) dispose();
        else unlisten = dispose;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}

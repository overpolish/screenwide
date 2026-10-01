// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useReducer } from "react";

import { reducePreviewZoom } from "./preview-zoom-state";

export function usePreviewZoom() {
  const [state, dispatch] = useReducer(reducePreviewZoom, { percent: 100 });
  const reportZoom = useCallback((percent: number) => {
    dispatch({ percent, source: "native" });
  }, []);
  const requestZoom = useCallback((percent: number) => {
    dispatch({ percent, source: "control" });
  }, []);
  return {
    reportZoom,
    requestZoom,
    zoomPercent: state.percent,
    zoomRequest: state.request,
  };
}

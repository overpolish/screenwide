// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type PreviewZoomRequest = { percent: number };

export type PreviewZoomState = {
  percent: number;
  request?: PreviewZoomRequest;
};

export type PreviewZoomAction = {
  percent: number;
  source: "native" | "control";
};

// Native reports update the readout only. Request identity changes exclusively
// for a control action, even when the user requests the same percentage again.
export function reducePreviewZoom(
  state: PreviewZoomState,
  action: PreviewZoomAction,
): PreviewZoomState {
  return {
    percent: action.percent,
    request:
      action.source === "control" ? { percent: action.percent } : state.request,
  };
}

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ScreenshotRegionExclusion } from "../recording-sources/api";
import type { Region } from "../recording-sources/types";

type NativeRect = { height: number; width: number; x: number; y: number };
type NativeHandle =
  | "body"
  | "east"
  | "north"
  | "northeast"
  | "northwest"
  | "south"
  | "southeast"
  | "southwest"
  | "west";
export type NativeGesture =
  "drawing" | "moving" | { resizing: { handle: NativeHandle } };
export type NativePayload = {
  gesture: NativeGesture | null;
  region: NativeRect | null;
  status: "cancelled" | "changed" | "finished" | "layout";
  monitorId?: number;
};
type ResizeDirection =
  | "bottom"
  | "bottomLeft"
  | "bottomRight"
  | "left"
  | "right"
  | "top"
  | "topLeft"
  | "topRight"
  | undefined;
export type NativeGestureState = {
  dragging: boolean;
  drawing: boolean;
  resizeDirection: ResizeDirection;
};
export type NativeScreenshotRegionOptions = {
  bounds: { height: number; width: number };
  enabled: boolean;
  onFinished: (
    region: Region,
    gesture: NativeGesture,
    monitorId?: number,
  ) => void;
  onGesture: (state: NativeGestureState) => void;
  onRegionChange: (region: Region) => void;
  region: Region;
  visible: boolean;
  windowLabel: string | undefined;
  allowDrawing?: boolean;
  aspect?: number;
  desktop?: boolean;
  exclusionRect?: ScreenshotRegionExclusion;
  inputEnabled?: boolean;
  monitorId?: number;
  onMonitorChange?: (monitorId: number) => void;
  onReconciled?: (region: Region, monitorId?: number) => void;
  showFrame?: boolean;
  showHandles?: boolean;
};

export const resizeDirections: Record<NativeHandle, ResizeDirection> = {
  body: undefined,
  east: "right",
  north: "top",
  northeast: "topRight",
  northwest: "topLeft",
  south: "bottom",
  southeast: "bottomRight",
  southwest: "bottomLeft",
  west: "left",
};
export const emptyRegion = (): Region => ({
  position: { x: 0, y: 0 },
  size: { height: 0, width: 0 },
});
export const nativeRegion = (rect: NativeRect | null): Region | null =>
  rect
    ? {
        position: { x: rect.x, y: rect.y },
        size: { height: rect.height, width: rect.width },
      }
    : null;

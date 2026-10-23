// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { MonitorDetails, MonitorThumbnail, WindowDetails } from "./types";

export const listMonitors = () => invoke<MonitorDetails[]>("list_monitors");

export const listMonitorThumbnails = () =>
  invoke<MonitorThumbnail[]>("list_monitor_thumbnails");

export const selectedWindowStatus = (window: WindowDetails) =>
  invoke<{ appIconPath: string | null; available: boolean }>(
    "selected_window_status",
    {
      id: window.id,
      pid: window.pid,
    },
  );

/** Starts picking the window to record by clicking it: the region overlay
 * cuts out the window under the pointer (`window-pick://hovered`), anchored
 * for the whole pick to `anchorMonitorId`, the monitor it is already on. */
export const startWindowPick = (anchorMonitorId: number | null) =>
  invoke<null>("start_window_pick", { anchorMonitorId });

/** Moves the pick through the stacked windows, for the keyboard. */
export const stepWindowPick = (delta: number) =>
  invoke<null>("step_window_pick", { delta });

/** Picks the window the pick is on (`window-pick://picked`); over none,
 * picking goes on. */
export const pickWindow = () => invoke<null>("pick_window");

/** Ends picking without a window. */
export const cancelWindowPick = () => invoke<null>("cancel_window_pick");

/** Puts every display's frozen desktop behind the overlay picking a window,
 * once the overlay is attached. */
export const showWindowPickBackdrop = () =>
  invoke<null>("show_window_pick_backdrop");

export const hideRecordingUi = () => invoke<null>("hide_recording_ui");

export const openRecordingUi = () => invoke<null>("open_recording_ui");

export const recordingUiVisible = () => invoke<boolean>("recording_ui_visible");

export const toggleRecordingUi = () => invoke<null>("toggle_recording_ui");

export const showRegionSelector = (monitor: MonitorDetails, desktop = false) =>
  invoke<null>("show_region_selector", {
    desktop,
    position: monitor.physicalPosition,
    size: monitor.physicalSize,
  });

export const hideRegionSelector = () => invoke<null>("hide_region_selector");

export const setRegionSelectorPassthrough = (passthrough: boolean) =>
  invoke<null>("set_region_selector_passthrough", { passthrough });

export const setRegionSelectorOpacity = (opacity: number) =>
  invoke<null>("set_region_selector_opacity", { opacity });

export const setScreenshotRegionSession = (
  active: boolean,
  restoreRegion = false,
) =>
  invoke<boolean>("set_screenshot_region_session", { active, restoreRegion });

export type ScreenshotRegionExclusion = {
  height: number;
  width: number;
  x: number;
  y: number;
};
export type ScreenshotRegionOscOptions = {
  bounds: { height: number; width: number };
  inputEnabled: boolean;
  region: { height: number; width: number; x: number; y: number };
  visible: boolean;
  window: string;
  allowDrawing?: boolean;
  aspect?: number;
  desktop?: boolean;
  exclusionRect?: ScreenshotRegionExclusion;
  monitorId?: number;
  showFrame?: boolean;
  showHandles?: boolean;
  /** The region is a window being picked: a rounded cutout, unframed. */
  windowHighlight?: boolean;
};

export const setScreenshotRegionOsc = (options: ScreenshotRegionOscOptions) =>
  invoke<boolean>("set_screenshot_region_osc", {
    allowDrawing: options.allowDrawing ?? true,
    aspect: options.aspect,
    desktop: options.desktop ?? false,
    exclusionRect: options.exclusionRect,
    height: options.region.height,
    inputEnabled: options.inputEnabled,
    monitorHeight: options.bounds.height,
    monitorId: options.monitorId,
    monitorWidth: options.bounds.width,
    showFrame: options.showFrame ?? true,
    showHandles: options.showHandles ?? true,
    visible: options.visible,
    width: options.region.width,
    window: options.window,
    windowHighlight: options.windowHighlight ?? false,
    x: options.region.x,
    y: options.region.y,
  });

export const setRecordingControlsBorrowed = (borrowed: boolean) =>
  invoke<null>("set_recording_controls_borrowed", { borrowed });

export const setRegionSelectorOscFrameVisible = (visible: boolean) =>
  invoke<boolean>("set_region_selector_osc_frame_visible", { visible });

export const beginRegionSelectorGesture = () =>
  invoke<null>("begin_region_selector_gesture");

export const finishRegionSelectorGesture = () =>
  invoke<null>("finish_region_selector_gesture");

export const prepareScreenshotRegionMagnifier = (
  monitorId: number,
  window: string,
) =>
  invoke<boolean>("prepare_screenshot_region_magnifier", {
    monitorId,
    window,
  });

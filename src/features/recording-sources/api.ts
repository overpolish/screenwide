// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { MonitorDetails, SelectorState, WindowDetails } from "./types";

export const listMonitors = () => invoke<MonitorDetails[]>("list_monitors");

export const listWindows = () => invoke<WindowDetails[]>("list_windows");

export const selectedWindowAvailable = (window: WindowDetails) =>
  invoke<boolean>("selected_window_available", {
    id: window.id,
    pid: window.pid,
  });

const windowIdentity = (window: WindowDetails) => ({
  id: window.id,
  pid: window.pid,
  title: window.title,
});

export const resizeWindow = (
  window: WindowDetails,
  width: number,
  height: number,
) =>
  invoke<null>("resize_window", {
    ...windowIdentity(window),
    height,
    width,
  });

export const expandRecordingSourceSelector = (
  windowSelector: boolean,
  focusContents: boolean,
) =>
  invoke<null>("expand_recording_source_selector", {
    focusContents,
    windowSelector,
  });

export const collapseRecordingSourceSelector = (returnFocus?: boolean) =>
  invoke<null>("collapse_recording_source_selector", { returnFocus });

export const getRecordingSourceSelectorState = () =>
  invoke<SelectorState>("get_recording_source_selector_state");

export const finishRecordingBarDrag = () =>
  invoke<null>("finish_recording_bar_drag");

export const hideRecordingUi = () => invoke<null>("hide_recording_ui");

export const recordingUiVisible = () => invoke<boolean>("recording_ui_visible");

export const toggleRecordingUi = () => invoke<null>("toggle_recording_ui");

export const setRecordingSourceSelectorVisible = (visible: boolean) =>
  invoke<null>("set_recording_source_selector_visible", { visible });

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
    x: options.region.x,
    y: options.region.y,
  });

export const setRecordingControlsOpacity = (opacity: number) =>
  invoke<null>("set_recording_controls_opacity", { opacity });

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

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel, invoke } from "@tauri-apps/api/core";

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

export const showRegionSelector = (monitor: MonitorDetails) =>
  invoke<null>("show_region_selector", {
    position: monitor.physicalPosition,
    size: monitor.physicalSize,
  });

export const hideRegionSelector = () => invoke<null>("hide_region_selector");

export const setRegionSelectorPassthrough = (passthrough: boolean) =>
  invoke<null>("set_region_selector_passthrough", { passthrough });

export const setRegionSelectorOpacity = (opacity: number) =>
  invoke<null>("set_region_selector_opacity", { opacity });

export const setScreenshotRegionSession = (active: boolean) =>
  invoke<null>("set_screenshot_region_session", { active });

export const openScreenshotRegionOverlays = (
  destination: "clipboard" | "export",
) => invoke<null>("open_screenshot_region_overlays", { destination });

export const closeScreenshotRegionOverlays = () =>
  invoke<null>("close_screenshot_region_overlays");

export const setRecordingControlsOpacity = (opacity: number) =>
  invoke<null>("set_recording_controls_opacity", { opacity });

export const beginRegionSelectorGesture = () =>
  invoke<null>("begin_region_selector_gesture");

export const finishRegionSelectorGesture = () =>
  invoke<null>("finish_region_selector_gesture");

export const takeMonitorScreenshot = (
  monitorId: number,
  channel: Channel<ArrayBuffer>,
) =>
  invoke<{ height: number; width: number }>("take_monitor_screenshot", {
    channel,
    monitorId,
  });

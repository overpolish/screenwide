// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentType } from "react";

type WindowLoader = () => Promise<ComponentType>;

/**
 * Each window's own code, loaded only by that window. Every window used to
 * load the whole app, and all of them load together at launch, so a window
 * that is shown early, such as an editor opened on a recovered recording,
 * waited on code for every other window first.
 *
 * `null` marks a transparent host for a native surface: no rendering or input
 * is allowed in React there, so it loads nothing at all.
 */
const WINDOW_ROUTES: Record<string, WindowLoader | null> = {
  "/alert": () =>
    import("../features/alert/alert-window").then(
      (module) => module.AlertWindow,
    ),
  "/annotate": null,
  "/annotate-toolbar": () =>
    import("../features/annotate/annotate-toolbar-window").then(
      (module) => module.AnnotateToolbarWindow,
    ),
  "/confirm-sheet": () =>
    import("../features/confirm-sheet/confirm-sheet-window").then(
      (module) => module.ConfirmSheetWindow,
    ),
  "/editor": () =>
    import("./editor-entry").then((module) => module.EditorEntry),
  "/export-options": () =>
    import("./export-options-entry").then(
      (module) => module.ExportOptionsEntry,
    ),
  "/glide": () =>
    import("../features/glide/glide-window").then(
      (module) => module.GlideWindow,
    ),
  "/glide-space": () =>
    import("../features/glide/glide-spaces-preview").then(
      (module) => module.GlideSpaceWindow,
    ),
  "/permissions": () =>
    import("./permissions-entry").then((module) => module.PermissionsEntry),
  "/qr-details": () =>
    import("../features/text-recognition/qr-details-window").then(
      (module) => module.QrDetailsWindow,
    ),
  "/recording-dock": () =>
    import("./recording-dock-entry").then(
      (module) => module.RecordingDockEntry,
    ),
  "/recording-source-selector": () =>
    import("./recording-source-selector-entry").then(
      (module) => module.RecordingSourceSelectorEntry,
    ),
  "/region-selector": () =>
    import("./region-selector-entry").then(
      (module) => module.RegionSelectorEntry,
    ),
  "/ruler": null,
  "/scrolling-capture-overlay": () =>
    import("../features/screenshots/scrolling-capture-overlay-window").then(
      (module) => module.ScrollingCaptureOverlayWindow,
    ),
  "/settings": () =>
    import("./settings-entry").then((module) => module.SettingsEntry),
  "/standalone-listbox": () =>
    import("./standalone-listbox-entry").then(
      (module) => module.StandaloneListboxEntry,
    ),
  "/text-recognition": null,
  "/tooltip": () =>
    import("../features/tooltip/tooltip-window").then(
      (module) => module.TooltipWindow,
    ),
  "/update": () =>
    import("../features/updates/update-prompt-window").then(
      (module) => module.UpdatePromptWindow,
    ),
};

/** The recording bar is the window at the root path. */
const loadRecordingBar: WindowLoader = () =>
  import("./recording-bar-entry").then((module) => module.RecordingBarEntry);

/** The window at `pathname`, or `null` for a native surface's host. */
export function loadWindow(pathname: string): Promise<ComponentType | null> {
  const route =
    pathname in WINDOW_ROUTES ? WINDOW_ROUTES[pathname] : loadRecordingBar;
  return route ? route() : Promise.resolve(null);
}

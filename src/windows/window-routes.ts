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
  "/glide-picker": () =>
    import("../features/glide/picker/glide-picker-window").then(
      (module) => module.GlidePickerWindow,
    ),
  "/island": () =>
    import("./island-entry").then((module) => module.IslandEntry),
  "/permissions": () =>
    import("./permissions-entry").then((module) => module.PermissionsEntry),
  "/projects": () =>
    import("./project-browser-entry").then(
      (module) => module.ProjectBrowserEntry,
    ),
  "/qr-details": () =>
    import("../features/text-recognition/qr-details-window").then(
      (module) => module.QrDetailsWindow,
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

/** The window at `pathname`, or `null` for a native surface's host or a
 * path no window has. */
export function loadWindow(pathname: string): Promise<ComponentType | null> {
  const route = pathname in WINDOW_ROUTES ? WINDOW_ROUTES[pathname] : null;
  return route ? route() : Promise.resolve(null);
}

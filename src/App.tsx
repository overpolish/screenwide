// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotateToolbarWindow } from "./features/annotate/annotate-toolbar-window";
import { ConfirmSheetWindow } from "./features/confirm-sheet/confirm-sheet-window";
import { EditorSync } from "./features/editor/editor-sync";
import { EditorWindow } from "./features/editor/editor-window";
import { ExportOptionsSync } from "./features/editor/export-options/export-options-sync";
import { ExportOptionsWindow } from "./features/editor/export-options/export-options-window";
import { ToolPanelSync } from "./features/editor/tool-panels/tool-panel-sync";
import { GlideSpaceWindow } from "./features/glide/glide-spaces-preview";
import { GlideWindow } from "./features/glide/glide-window";
import { PermissionSync } from "./features/permissions/permission-sync";
import { PermissionsWindow } from "./features/permissions/permissions-window";
import { PopupPanelSync } from "./features/popup-panel/popup-panel-sync";
import { PopupPanelWindow } from "./features/popup-panel/popup-panel-window";
import { RecordingBarWindow } from "./features/recording-controls/components/recording-bar-window";
import { RecordingDockWindow } from "./features/recording-controls/components/recording-dock-window";
import { RecordingStateSync } from "./features/recording-controls/recording-state-sync";
import { RecordingInputSync } from "./features/recording-inputs/recording-input-sync";
import { RecordingSourceSelectorWindow } from "./features/recording-sources/recording-source-selector-window";
import { RecordingSourceSync } from "./features/recording-sources/recording-source-sync";
import { RegionSelectorWindow } from "./features/region-selector/region-selector-window";
import { ScrollingCaptureOverlayWindow } from "./features/screenshots/scrolling-capture-overlay-window";
import { SettingsWindow } from "./features/settings/settings-window";
import { QrDetailsWindow } from "./features/text-recognition/qr-details-window";
import { TooltipWindow } from "./features/tooltip/tooltip-window";
import { UpdatePromptWindow } from "./features/updates/update-prompt-window";

export function App() {
  const content = (() => {
    switch (window.location.pathname) {
      case "/annotate":
        // The native annotate overlay uses this route only as its transparent
        // host. No annotation rendering or interaction is allowed in React.
        return null;
      case "/annotate-toolbar":
        return <AnnotateToolbarWindow />;
      case "/confirm-sheet":
        return <ConfirmSheetWindow />;
      case "/editor":
        return <EditorWindow />;
      case "/export-options":
        return <ExportOptionsWindow />;
      case "/glide":
        return <GlideWindow />;
      case "/glide-space":
        return <GlideSpaceWindow />;
      case "/permissions":
        return <PermissionsWindow />;
      case "/recording-dock":
        return <RecordingDockWindow />;
      case "/recording-source-selector":
        return <RecordingSourceSelectorWindow />;
      case "/region-selector":
        return <RegionSelectorWindow />;
      case "/ruler":
        // Native ruler surfaces use this route only as their transparent host.
        // No ruler rendering or interaction is allowed in React.
        return null;
      case "/scrolling-capture-overlay":
        return <ScrollingCaptureOverlayWindow />;
      case "/settings":
        return <SettingsWindow />;
      case "/standalone-listbox":
        return <PopupPanelWindow />;
      case "/tooltip":
        return <TooltipWindow />;
      case "/text-recognition":
        // Native capture surfaces use this transparent webview only as their
        // platform host. It deliberately has no React overlay or input path.
        return null;
      case "/qr-details":
        return <QrDetailsWindow />;
      case "/update":
        return <UpdatePromptWindow />;
      default:
        return <RecordingBarWindow />;
    }
  })();

  return (
    <>
      <EditorSync />
      <ExportOptionsSync />
      <ToolPanelSync />
      <PermissionSync />
      <RecordingInputSync />
      <RecordingSourceSync />
      <RecordingStateSync />
      <PopupPanelSync />
      {content}
    </>
  );
}

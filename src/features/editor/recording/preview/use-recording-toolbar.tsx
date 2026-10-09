// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { useCallback, useMemo } from "react";

import { ButtonGroup } from "../../../../components/base/button-group/button-group";
import { t } from "../../../../i18n/i18n";
import { useProvideEditorToolbarTools } from "../../panel/editor-toolbar-context";
import { CursorToolToggle } from "../../tool-panels/cursor/cursor-tool-toggle";
import { KeyboardToolToggle } from "../../tool-panels/keyboard/keyboard-tool-toggle";
import { useToolPanel } from "../../tool-panels/use-tool-panel";

import {
  RecordingCanvasTools,
  RecordingCanvasTool,
} from "./recording-crop-toggle";
export function useRecordingToolbar({
  canEditActiveTrack,
  canResizeActiveTrack,
  canvasTool,
  changeCanvasTool,
  hasAnnotations,
  hasCursorData,
  hasKeyboardData,
  hasScenes,
  hasVisiblePanes,
}: {
  canEditActiveTrack: boolean;
  canResizeActiveTrack: boolean;
  canvasTool: RecordingCanvasTool;
  changeCanvasTool: (tool: RecordingCanvasTool) => void;
  /** Whether annotations can be drawn, which the annotation tools are for. */
  hasAnnotations: boolean;
  hasCursorData: boolean;
  hasKeyboardData: boolean;
  /** Whether the recording has a screen and a camera to arrange. */
  hasScenes: boolean;
  hasVisiblePanes: boolean;
}) {
  const {
    close: closeToolPanel,
    openTool: openToolPanel,
    toggle: toggleToolPanel,
  } = useToolPanel("recording");
  // Cursor and Keyboard are panels without a canvas tool behind them, so
  // putting one away closes the panel and leaves no tool in hand, the way
  // pressing an active canvas tool does.
  const dismissToolPanel = useCallback(() => {
    void closeToolPanel();
  }, [closeToolPanel]);
  // The shortcut opens the panel from the toolbar button's own bounds, the
  // same anchor a press would give it.
  const togglePanel = useCallback(
    (panel: "cursor" | "keyboard") => {
      if (openToolPanel === panel) {
        dismissToolPanel();
        return;
      }
      const bounds = document
        .querySelector(`[data-editor-tool=${panel}]`)
        ?.getBoundingClientRect();
      if (bounds) void toggleToolPanel(panel, bounds);
    },
    [dismissToolPanel, openToolPanel, toggleToolPanel],
  );
  const toggleCursorPanel = useCallback(() => {
    togglePanel("cursor");
  }, [togglePanel]);
  const toggleKeyboardPanel = useCallback(() => {
    togglePanel("keyboard");
  }, [togglePanel]);
  // The tools read the committed `recordingOutput`, never the resize draft, so
  // holding the element keeps the title bar's tools stable mid-gesture. Held as
  // one element so the bar re-renders only when a tool actually changes.
  const cropToggle = useMemo(
    () =>
      hasVisiblePanes ? (
        <>
          {hasCursorData || hasKeyboardData ? (
            <ButtonGroup
              aria-label={t("editor-recording-effects")}
              className="gap-control"
            >
              {hasCursorData ? (
                <CursorToolToggle onDismiss={dismissToolPanel} />
              ) : null}
              {hasKeyboardData ? (
                <KeyboardToolToggle onDismiss={dismissToolPanel} />
              ) : null}
            </ButtonGroup>
          ) : null}
          <RecordingCanvasTools
            hasAnnotations={hasAnnotations}
            hasScenes={hasScenes}
            isArrowEnabled={hasVisiblePanes}
            isEnabled={canEditActiveTrack}
            isFrameEnabled={canResizeActiveTrack}
            isSelectEnabled={hasVisiblePanes}
            onToolChange={changeCanvasTool}
            tool={canvasTool}
          />
        </>
      ) : undefined,
    [
      canEditActiveTrack,
      canResizeActiveTrack,
      hasVisiblePanes,
      canvasTool,
      changeCanvasTool,
      dismissToolPanel,
      hasAnnotations,
      hasCursorData,
      hasKeyboardData,
      hasScenes,
    ],
  );
  // The tools belong to the title bar above, the way a unified toolbar carries
  // them. They are offered only while there is a picture to point them at.
  useProvideEditorToolbarTools(hasVisiblePanes ? cropToggle : null);
  return { toggleCursorPanel, toggleKeyboardPanel };
}

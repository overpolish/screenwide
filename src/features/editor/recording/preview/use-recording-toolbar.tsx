// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { useCallback, useMemo } from "react";

import { t } from "../../../../i18n/i18n";
import { Toolbar } from "../../../../primitives/toolbar/toolbar";
import { useProvideEditorToolbarTools } from "../../panel/editor-toolbar-context";
import { CaptionsToolToggle } from "../../tool-panels/captions/captions-tool-toggle";
import { CursorToolToggle } from "../../tool-panels/cursor/cursor-tool-toggle";
import { KeyboardToolToggle } from "../../tool-panels/keyboard/keyboard-tool-toggle";
import { TranscriptToolToggle } from "../../tool-panels/transcript/transcript-tool-toggle";
import { useToolPanel } from "../../tool-panels/use-tool-panel";

import { CAPTIONS_LAYER_ID, OverlayLayerId } from "./overlay-layers";
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
  hasMicrophone,
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
  /** Whether the recording has a microphone track, which the Transcript
   * tool transcribes and the Captions tool captions. */
  hasMicrophone: boolean;
  /** Whether the recording has a screen and a camera to arrange. */
  hasScenes: boolean;
  hasVisiblePanes: boolean;
}) {
  const {
    close: closeToolPanel,
    openTool: openToolPanel,
    toggle: toggleToolPanel,
  } = useToolPanel("recording");
  // Cursor, Keyboard and Transcript are panels without a canvas tool behind
  // them, so putting one away closes the panel and leaves no tool in hand,
  // the way pressing an active canvas tool does.
  const dismissToolPanel = useCallback(() => {
    void closeToolPanel();
  }, [closeToolPanel]);
  // The shortcut opens the panel from the toolbar button's own bounds, the
  // same anchor a press would give it.
  const togglePanel = useCallback(
    (panel: "captions" | "cursor" | "keyboard" | "transcript") => {
      if (openToolPanel === panel) {
        dismissToolPanel();
        return;
      }
      const bounds = document
        .querySelector(`[data-editor-tool=${panel}]`)
        ?.getBoundingClientRect();
      // With no picture there is nothing for the window to grow for.
      if (bounds)
        void toggleToolPanel(
          panel,
          bounds,
          hasVisiblePanes ? undefined : false,
        );
    },
    [dismissToolPanel, hasVisiblePanes, openToolPanel, toggleToolPanel],
  );
  const toggleCursorPanel = useCallback(() => {
    togglePanel("cursor");
  }, [togglePanel]);
  const toggleKeyboardPanel = useCallback(() => {
    togglePanel("keyboard");
  }, [togglePanel]);
  const toggleTranscriptPanel = useCallback(() => {
    togglePanel("transcript");
  }, [togglePanel]);
  const toggleCaptionsPanel = useCallback(() => {
    togglePanel("captions");
  }, [togglePanel]);
  // The tools read the committed `recordingOutput`, never the resize draft, so
  // holding the element keeps the title bar's tools stable mid-gesture. Held as
  // one element so the bar re-renders only when a tool actually changes.
  const cropToggle = useMemo(
    () =>
      hasVisiblePanes ? (
        <>
          {hasCursorData || hasKeyboardData || hasMicrophone ? (
            <Toolbar aria-label={t("editor-recording-effects")}>
              {hasCursorData ? (
                <CursorToolToggle onDismiss={dismissToolPanel} />
              ) : null}
              {hasKeyboardData ? (
                <KeyboardToolToggle onDismiss={dismissToolPanel} />
              ) : null}
              {hasMicrophone ? (
                <>
                  <TranscriptToolToggle onDismiss={dismissToolPanel} />
                  <CaptionsToolToggle onDismiss={dismissToolPanel} />
                </>
              ) : null}
            </Toolbar>
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
      ) : hasMicrophone ? (
        // The transcript and its captions are the voice's, not the picture's,
        // so they are offered with no picture to point the other tools at: an
        // audio recording, or one whose video tracks are all turned off.
        <Toolbar aria-label={t("editor-recording-effects")}>
          <TranscriptToolToggle
            onDismiss={dismissToolPanel}
            resetsView={false}
          />
          <CaptionsToolToggle onDismiss={dismissToolPanel} resetsView={false} />
        </Toolbar>
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
      hasMicrophone,
      hasScenes,
    ],
  );
  // The tools belong to the title bar above, the way a unified toolbar carries
  // them. The picture's tools are offered only while there is a picture to
  // point them at; the Transcript tool wherever there is a microphone.
  useProvideEditorToolbarTools(cropToggle ?? null);
  // A chosen overlay asks for its panel rather than toggling it.
  const openOverlayTool = useCallback(
    (layerId: OverlayLayerId) => {
      const panel = layerId === CAPTIONS_LAYER_ID ? "captions" : "keyboard";
      if (openToolPanel !== panel) togglePanel(panel);
    },
    [openToolPanel, togglePanel],
  );
  // A panel's shortcut does nothing where the recording has nothing for it.
  return {
    openOverlayTool,
    panelToggles: {
      toggleCaptionsPanel: hasMicrophone ? toggleCaptionsPanel : undefined,
      toggleCursorPanel: hasCursorData ? toggleCursorPanel : undefined,
      toggleKeyboardPanel: hasKeyboardData ? toggleKeyboardPanel : undefined,
      toggleTranscriptPanel: hasMicrophone ? toggleTranscriptPanel : undefined,
    },
  };
}

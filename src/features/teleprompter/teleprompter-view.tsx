// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useRef } from "react";

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import { Alert } from "../../primitives/alert/alert";
import { SkinContext } from "../../primitives/skin/skin";
import { Text } from "../../primitives/text/text";
import { IslandModelLine } from "../editor/island/island-model-line";
import { type PopupMenuAnchor } from "../popup-panel/use-popup-menu";

import { type CardCue, type CardPlace, CueCard } from "./cues/cue-card";
import { type MarkedCue } from "./cues/cue-marks";
import { type CuePlace } from "./cues/place-cue";
import { ScriptText } from "./script-words";
import { TeleprompterBar, type TeleprompterBarProps } from "./teleprompter-bar";
import { TeleprompterText } from "./teleprompter-text";
import { VoiceState } from "./use-teleprompter-reading";
import { type TextFrame, useTextFrame } from "./use-text-frame";

import type { ToolPanelModel } from "../editor/tools/tool-panel-selection";

export type TeleprompterViewProps = Omit<TeleprompterBarProps, "isSteady"> & {
  /** Where the cue shown stands in the window. */
  cuePlace: CardPlace;
  /** The cues, marked beside their words. */
  cues: readonly MarkedCue[];
  /** Cues dragged or pushed to other words: each moved one, at its word. */
  onCuesMove: (moved: readonly CuePlace[]) => void;
  onDownloadVoiceModel: () => void;
  onEdit: (text: string) => void;
  /** The word starting `at` picked for the cue being placed. */
  onPlace: (at: number) => void;
  onScrollTo: (index: number) => void;
  onWordMenu: (anchor: PopupMenuAnchor, at: number) => void;
  /** The next word to read, fractional between words. */
  place: number;
  /** A cue from an editor waiting for its word, with its picture once
   * drawn. */
  placing: { picture: ImageBitmap | string | null } | null;
  script: ScriptText;
  /** The cue shown in the window; none between cues, or while it stands
   * beside the window. */
  shownCue: CardCue | null;
  /** The script as written. */
  text: string;
  voice: VoiceState;
  /** Dictation's model, which the voice is heard with; null until known. */
  voiceModel: ToolPanelModel | null;
  /** Where the text frame stands, as it opens and changes. */
  onTextFrameChange?: (frame: TextFrame) => void;
  /** Set when the last change could not be saved. */
  saveError?: string;
};

/** What stands under the bar about a cue being placed, the voice or
 * saving, if anything: the voice's model fetched from right here while
 * following the voice needs it, as every island's models are. */
function Note({
  isPlacing,
  needsModel,
  onDownloadVoiceModel,
  saveError,
  voice,
  voiceModel,
}: Pick<
  TeleprompterViewProps,
  "onDownloadVoiceModel" | "saveError" | "voice" | "voiceModel"
> & { isPlacing: boolean; needsModel: boolean }) {
  if (saveError)
    return (
      <Alert color="error" role="alert">
        {t("teleprompter-save-failed")}
      </Alert>
    );
  if (isPlacing)
    return (
      <Text className="ps-inset-sm text-fg-secondary" variant="caption">
        {t("teleprompter-cue-placing")}
      </Text>
    );
  if (needsModel && voiceModel)
    return (
      <IslandModelLine
        model={voiceModel}
        onDownload={onDownloadVoiceModel}
        title={t("teleprompter-follow-voice")}
      />
    );
  if (voice.kind === "loading")
    return (
      <Text className="ps-inset-sm text-fg-secondary" variant="caption">
        {t("teleprompter-voice-loading")}
      </Text>
    );
  if (voice.kind !== "problem" || voice.problem === "noModel") return null;
  return (
    <Alert color="error">
      {voice.problem === "noMicrophone"
        ? t("teleprompter-voice-no-microphone")
        : t("teleprompter-voice-failed")}
    </Alert>
  );
}

// The script is read on a line near the top, so the eyes stay by the
// camera. While it plays the bar gives way to the text until the pointer
// comes back to the window. Mirrored, the whole window reads through a
// teleprompter's glass, its controls included; filling the display it
// loses its corners.
export function TeleprompterView({
  cuePlace,
  cues,
  onCuesMove,
  onDownloadVoiceModel,
  onEdit,
  onPlace,
  onScrollTo,
  onTextFrameChange,
  onWordMenu,
  place,
  placing,
  saveError,
  script,
  shownCue,
  text,
  voice,
  voiceModel,
  ...bar
}: TeleprompterViewProps) {
  const isSteady =
    bar.preferences.mode === "steady" || voice.kind === "problem";
  const frameRef = useRef<HTMLDivElement>(null);
  useTextFrame(frameRef, onTextFrameChange);
  return (
    <SkinContext value="island">
      <div
        className={cn(
          "group dark flex size-full flex-col overflow-hidden bg-island text-fg",
          !bar.isFullscreen &&
            "rounded-island inset-ring inset-ring-fill-strong",
        )}
        style={{
          transform: bar.preferences.isMirrored ? "scaleX(-1)" : undefined,
        }}
      >
        <div
          className={cn(
            "flex flex-col gap-inline pt-inset-sm transition-opacity duration-200",
            bar.isPlaying &&
              "opacity-0 group-hover:opacity-100 focus-within:opacity-100",
          )}
        >
          <TeleprompterBar {...bar} isSteady={isSteady} />
          <div className="px-inset-sm empty:hidden">
            <Note
              isPlacing={placing !== null}
              needsModel={
                bar.preferences.mode === "voice" &&
                voiceModel !== null &&
                voiceModel.status !== "downloaded"
              }
              onDownloadVoiceModel={onDownloadVoiceModel}
              saveError={saveError}
              voice={voice}
              voiceModel={voiceModel}
            />
          </div>
        </div>
        <div className="relative flex min-h-0 grow flex-col" ref={frameRef}>
          <TeleprompterText
            columnWidth={bar.preferences.columnWidth}
            cues={cues}
            isPlaying={bar.isPlaying}
            onColumnWidthChange={(columnWidth) => {
              bar.onPreferencesChange({ columnWidth });
            }}
            onCuesMove={onCuesMove}
            onEdit={onEdit}
            onPlace={onPlace}
            onScrollTo={onScrollTo}
            onWordMenu={onWordMenu}
            place={place}
            placing={placing}
            script={script}
            text={text}
            textSize={bar.preferences.textSize}
          />
          <CueCard cue={shownCue} place={cuePlace} />
        </div>
      </div>
    </SkinContext>
  );
}

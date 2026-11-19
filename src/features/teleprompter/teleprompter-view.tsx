// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import { Alert } from "../../primitives/alert/alert";
import { SkinContext } from "../../primitives/skin/skin";
import { Text } from "../../primitives/text/text";
import { IslandModelLine } from "../editor/island/island-model-line";

import { ScriptText } from "./script-words";
import { TeleprompterBar, type TeleprompterBarProps } from "./teleprompter-bar";
import { TeleprompterText } from "./teleprompter-text";
import { VoiceState } from "./use-teleprompter-reading";

import type { ToolPanelModel } from "../editor/tools/tool-panel-selection";

export type TeleprompterViewProps = Omit<TeleprompterBarProps, "isSteady"> & {
  onDownloadVoiceModel: () => void;
  onEdit: (text: string) => void;
  onScrollTo: (index: number) => void;
  /** The next word to read, fractional between words. */
  place: number;
  script: ScriptText;
  /** The script as written. */
  text: string;
  voice: VoiceState;
  /** Dictation's model, which the voice is heard with; null until known. */
  voiceModel: ToolPanelModel | null;
  /** Set when the last change could not be saved. */
  saveError?: string;
};

/** What stands under the bar about the voice or saving, if anything: the
 * voice's model fetched from right here while following the voice needs
 * it, as every island's models are. */
function Note({
  needsModel,
  onDownloadVoiceModel,
  saveError,
  voice,
  voiceModel,
}: Pick<
  TeleprompterViewProps,
  "onDownloadVoiceModel" | "saveError" | "voice" | "voiceModel"
> & { needsModel: boolean }) {
  if (saveError)
    return (
      <Alert color="error" role="alert">
        {t("teleprompter-save-failed")}
      </Alert>
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
  onDownloadVoiceModel,
  onEdit,
  onScrollTo,
  place,
  saveError,
  script,
  text,
  voice,
  voiceModel,
  ...bar
}: TeleprompterViewProps) {
  const isSteady =
    bar.preferences.mode === "steady" || voice.kind === "problem";
  return (
    <SkinContext value="island">
      <div
        className={cn(
          "group dark flex size-full flex-col overflow-hidden bg-island text-fg",
          !bar.isFullscreen &&
            "rounded-panel inset-ring inset-ring-fill-strong",
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
        <TeleprompterText
          columnWidth={bar.preferences.columnWidth}
          isPlaying={bar.isPlaying}
          onColumnWidthChange={(columnWidth) => {
            bar.onPreferencesChange({ columnWidth });
          }}
          onEdit={onEdit}
          onScrollTo={onScrollTo}
          place={place}
          script={script}
          text={text}
          textSize={bar.preferences.textSize}
        />
      </div>
    </SkinContext>
  );
}

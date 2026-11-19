// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import { Alert } from "../../primitives/alert/alert";
import { Button } from "../../primitives/button/button";
import { SkinContext } from "../../primitives/skin/skin";
import { Text } from "../../primitives/text/text";

import { ScriptText } from "./script-words";
import { TeleprompterBar, type TeleprompterBarProps } from "./teleprompter-bar";
import { TeleprompterText } from "./teleprompter-text";
import { VoiceState } from "./use-teleprompter-reading";

export type TeleprompterViewProps = Omit<TeleprompterBarProps, "isSteady"> & {
  /** The script as written. */
  markdown: string;
  onEdit: (markdown: string) => void;
  onOpenModels: () => void;
  onScrollTo: (index: number) => void;
  /** The next word to read, fractional between words. */
  place: number;
  script: ScriptText;
  voice: VoiceState;
  /** Set when the last change could not be saved. */
  saveError?: string;
};

/** What stands under the bar about the voice or saving, if anything. */
function Note({
  onOpenModels,
  saveError,
  voice,
}: Pick<TeleprompterViewProps, "onOpenModels" | "saveError" | "voice">) {
  if (saveError)
    return (
      <Alert color="error" role="alert">
        {t("teleprompter-save-failed")}
      </Alert>
    );
  if (voice.kind === "loading")
    return (
      <Text className="ps-inset-sm text-fg-secondary" variant="caption">
        {t("teleprompter-voice-loading")}
      </Text>
    );
  if (voice.kind !== "problem") return null;
  if (voice.problem === "noModel")
    return (
      <Alert
        action={
          <Button onPress={onOpenModels} variant="ghost">
            {t("teleprompter-voice-download")}
          </Button>
        }
      >
        {t("teleprompter-voice-no-model")}
      </Alert>
    );
  return (
    <Alert color="error">
      {voice.problem === "noMicrophone"
        ? t("teleprompter-voice-no-microphone")
        : t("teleprompter-voice-failed")}
    </Alert>
  );
}

/**
 * The teleprompter: its bar, then the script, large, on the island's
 * black, read on a line near the top so the eyes stay by the camera. While
 * it plays the bar gives way to the text until the pointer comes back to
 * the window. Mirrored, the whole window reads through a teleprompter's
 * glass, its controls included; filling the display it loses its corners.
 */
export function TeleprompterView({
  markdown,
  onEdit,
  onOpenModels,
  onScrollTo,
  place,
  saveError,
  script,
  voice,
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
              onOpenModels={onOpenModels}
              saveError={saveError}
              voice={voice}
            />
          </div>
        </div>
        <TeleprompterText
          columnWidth={bar.preferences.columnWidth}
          isPlaying={bar.isPlaying}
          markdown={markdown}
          onColumnWidthChange={(columnWidth) => {
            bar.onPreferencesChange({ columnWidth });
          }}
          onEdit={onEdit}
          onScrollTo={onScrollTo}
          place={place}
          script={script}
          textSize={bar.preferences.textSize}
        />
      </div>
    </SkinContext>
  );
}

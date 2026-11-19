// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMemo, useState } from "react";

import { useModel } from "../editor/recording/microphone/use-model";
import { renameProject } from "../project-browser/api";
import { RecordingStateSync } from "../recording-controls/recording-state-sync";

import { setTeleprompterFullscreen } from "./api";
import { placeAfterEdit, scriptText } from "./script-words";
import {
  stepped,
  TEXT_SIZES,
  useTeleprompterPreferences,
} from "./teleprompter-preferences";
import { TeleprompterView } from "./teleprompter-view";
import { useRecordingLink } from "./use-recording-link";
import { useScriptSaving } from "./use-script-saving";
import { useTeleprompterKeys } from "./use-teleprompter-keys";
import { useTeleprompterReading } from "./use-teleprompter-reading";
import {
  type ShownScript,
  useTeleprompterScript,
} from "./use-teleprompter-script";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

// The teleprompter's window (`src-tauri/src/teleprompter/window.rs`): the
// script it was opened on, written and read here, at a steady pace or
// following the voice, playing and pausing with a recording. Another
// script opened in it is read from its start, paused.
export function TeleprompterWindow() {
  const shown = useTeleprompterScript();
  return (
    <>
      <RecordingStateSync />
      {/* The island's black alone until the script has been read. */}
      {shown ? (
        <ScriptReader key={shown.opened} shown={shown} />
      ) : (
        <div className="size-full rounded-panel bg-island" />
      )}
    </>
  );
}

function ScriptReader({ shown }: { shown: ShownScript }) {
  const { change, ...preferences } = useTeleprompterPreferences();
  const [text, setText] = useState(shown.script.text);
  const [title, setTitle] = useState(shown.script.title);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const script = useMemo(() => scriptText(text), [text]);
  const saving = useScriptSaving(shown.project);
  // The voice is heard with dictation's model, which can be fetched from
  // here; listening starts again once it has arrived.
  const voiceModel = useModel("dictation");
  const { place, setPlace, voice } = useTeleprompterReading({
    hasModel: voiceModel.shown?.status === "downloaded",
    isPlaying,
    mode: preferences.mode,
    spoken: script.spoken,
    wordsPerMinute: preferences.wordsPerMinute,
  });
  useRecordingLink(setIsPlaying);

  const close = () => {
    void saving
      .flush()
      .finally(() =>
        getCurrentWindow()
          .close()
          .catch(report("Could not close the teleprompter")),
      );
  };
  const fullscreen = (next: boolean) => {
    setTeleprompterFullscreen(next)
      .then(() => {
        setIsFullscreen(next);
      })
      .catch(report("Could not change the teleprompter's size"));
  };
  const textSize = (steps: number) => {
    change({ textSize: stepped(preferences.textSize, steps, TEXT_SIZES) });
  };
  useTeleprompterKeys({
    isFullscreen,
    isPlaying,
    onClose: close,
    onFullscreenChange: fullscreen,
    onPlace: setPlace,
    onPlayingChange: setIsPlaying,
    onTextSize: textSize,
    place,
  });

  return (
    <TeleprompterView
      isFullscreen={isFullscreen}
      isPlaying={isPlaying}
      onClose={close}
      onDownloadVoiceModel={voiceModel.download}
      onEdit={(next) => {
        // Writing pauses the text, so it does not move under the caret.
        setIsPlaying(false);
        setPlace(
          placeAfterEdit(
            { script, text },
            { script: scriptText(next), text: next },
            place,
          ),
        );
        setText(next);
        saving.save(next);
      }}
      onFullscreenChange={fullscreen}
      onPlayingChange={setIsPlaying}
      onPreferencesChange={change}
      onRename={(next) => {
        // What waits is written where the script is now, before it moves;
        // the move comes back as the project's new manifest.
        void saving
          .flush()
          .then(() => renameProject(shown.project, next))
          .then(() => {
            setTitle(next);
          })
          .catch(report("Could not rename the script"));
      }}
      onRestart={() => {
        setPlace(0);
      }}
      onScrollTo={setPlace}
      place={place}
      preferences={preferences}
      saveError={saving.error}
      script={script}
      text={text}
      title={title}
      voice={voice}
      voiceModel={voiceModel.shown}
    />
  );
}

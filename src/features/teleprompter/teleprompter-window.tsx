// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useMemo, useState } from "react";

import { useModel } from "../editor/recording/microphone/use-model";
import { PopupPanelSync } from "../popup-panel/popup-panel-sync";
import { renameProject } from "../project-browser/api";
import { RecordingStateSync } from "../recording-controls/recording-state-sync";

import { setTeleprompterFullscreen } from "./api";
import { shownCue } from "./cues/shown-cue";
import { useCueMenus } from "./cues/use-cue-menus";
import { useCuePlace } from "./cues/use-cue-place";
import { type PlacingCue, useCuePlacing } from "./cues/use-cue-placing";
import { useCues } from "./cues/use-cues";
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
  // Kept above the script read, so a cue waiting for its word stays as the
  // script it is for opens in place of another.
  const { place, placing } = useCuePlacing();
  return (
    <>
      <RecordingStateSync />
      <PopupPanelSync />
      {/* The island's black alone until the script has been read. */}
      {shown ? (
        <ScriptReader
          key={shown.opened}
          onPlace={place}
          placing={placing}
          shown={shown}
        />
      ) : (
        <div className="size-full rounded-island bg-island" />
      )}
    </>
  );
}

function ScriptReader({
  onPlace,
  placing,
  shown,
}: {
  /** Places the cue waiting for its word at the word starting `at` in the
   * script project `project`. */
  onPlace: (at: number, project: string) => void;
  placing: PlacingCue | null;
  shown: ShownScript;
}) {
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
  const cues = useCues(shown.project);
  const cueMenu = useCueMenus({
    cues: cues.cues,
    onRemove: cues.remove,
    script: shown.project,
  });
  const reached = shownCue(cues.cues, script, place);
  const cuePlace = useCuePlace(reached, preferences);
  const marked = useMemo(
    () =>
      cues.cues.map(({ at, id, path, title }) => ({
        at,
        id,
        src: convertFileSrc(path),
        title,
      })),
    [cues.cues],
  );

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
      cuePlace={cuePlace.place}
      cues={marked}
      isFullscreen={isFullscreen}
      isPlaying={isPlaying}
      onClose={close}
      onCuesMove={cues.move}
      onDownloadVoiceModel={voiceModel.download}
      onEdit={(next) => {
        // Writing pauses the text, so it does not move under the caret.
        setIsPlaying(false);
        const nextScript = scriptText(next);
        setPlace(
          placeAfterEdit(
            { script, text },
            { script: nextScript, text: next },
            place,
          ),
        );
        cues.edited(text, next, nextScript);
        setText(next);
        saving.save(next);
      }}
      onFullscreenChange={fullscreen}
      onPlace={(at) => {
        onPlace(at, shown.project);
      }}
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
      onTextFrameChange={cuePlace.onTextFrameChange}
      onWordMenu={cueMenu}
      place={place}
      placing={placing}
      preferences={preferences}
      saveError={saving.error}
      script={script}
      shownCue={cuePlace.drawn}
      text={text}
      title={title}
      voice={voice}
      voiceModel={voiceModel.shown}
    />
  );
}

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useModel } from "../../recording/microphone/use-model";
import {
  MOST_PREVIEW_MS,
  previewSound,
  useClipSounds,
} from "../../sound-track/sound-library";
import { useSingingProgress } from "../../sound-track/use-singing-progress";
import { useIslandSoundPlayback } from "../../timeline/moments/use-note-playback";

import type { ToolPanelModel } from "../../tools/tool-panel-selection";
import type { ToolPanelStripItem } from "../../tools/tool-panel-strip";
import type { SoundPreview } from "../audio-clip/sound-start-picker";

// The chosen song or sound effect as its island hears and draws it: its
// sound's waveform, from the library, and the clip played from where it
// starts, in the island's own window as a moment's voice note is, the
// editor pausing while it plays. Moving where it starts loads the stretch
// it now plays. A song played instrumental needs Studio ducking's model,
// offered here, and shows how far its singing is separated.

export type AudioClipPreviewState = {
  /** How loud the chosen clip's sound is along it, or nothing while
   * nothing is chosen. */
  levels: number[];
  /** The model that takes the singing out, null until the app has said. */
  model: ToolPanelModel | null;
  onDownloadModel: () => void;
  preview: SoundPreview;
  /** How far its singing is separated, or null while none is under way. */
  separating: number | null;
};

export function useAudioClipPreview(
  item: ToolPanelStripItem | null | undefined,
): AudioClipPreviewState {
  const clip = item?.kind === "audioClip" ? item : null;
  const sounds = useClipSounds(clip ? [clip.sound] : []);
  const model = useModel("studioDucking");
  const separating = useSingingProgress(clip?.sound ?? null);
  const lengthMs = clip ? Math.min(clip.clipMs, MOST_PREVIEW_MS) : 0;
  const playback = useIslandSoundPlayback(
    clip ? `${clip.sound}:${String(clip.offsetMs)}:${String(lengthMs)}` : null,
    () =>
      clip
        ? previewSound(clip.sound, clip.offsetMs, lengthMs)
        : Promise.reject(new Error("No clip is chosen")),
  );
  return {
    levels: (clip && sounds.get(clip.sound)?.levels) ?? [],
    model: model.shown,
    onDownloadModel: model.download,
    preview: {
      isPlaying: playback.playing,
      lengthMs,
      onTogglePlay: () => {
        void playback.toggle();
      },
      progress: playback.progress,
    },
    separating,
  };
}

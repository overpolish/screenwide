// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioLines, Music, Trash2 } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { LoopToggle } from "../../../annotations/options/loop-toggle";
import { LEAST_CLIP_MS } from "../../sound-track/audio-clips";
import { AnnotationOptions } from "../annotation/annotation-island-parts";

import {
  AudioClipVolumeRow,
  InstrumentalNote,
  InstrumentalToggle,
} from "./audio-clip-parts";
import { type SoundPreview, SoundStartPicker } from "./sound-start-picker";

import type { ToolPanelModel } from "../../tools/tool-panel-selection";
import type { EditorIslandContent } from "../editor-island";

// A song or sound effect chosen on the strip, in hand: its title, and
// deleting it, on the row. Under it the sound's waveform, where the clip's
// stretch is chosen and heard, how loud it plays, from the level its kind
// plays at by default, whether it loops for as long as its clip runs and,
// for a song, whether it plays without its singing; giving it another
// sound is a one-shot action at the end. An instrumental song needs Studio
// ducking's model, offered under its toggle, and says while its singing is
// separated. Its fades are its kind's own, songs that touch crossfade, and
// songs make way for the voice.

/** What the island asks of the clip in hand. */
export type AudioClipChange = {
  delete?: true;
  isInstrumental?: boolean;
  isLooped?: boolean;
  /** Play its sound from this far in, in ms. */
  offsetMs?: number;
  replace?: true;
  volumeDb?: number;
};

export type AudioClipIslandProps = {
  /** How long the clip plays, in ms. */
  clipMs: number;
  id: string;
  /** Whether a song plays without its singing. */
  isInstrumental: boolean;
  isLooped: boolean;
  isSong: boolean;
  /** How loud its sound is, from 0 to 1, evenly along it. */
  levels: number[];
  /** The model that takes the singing out, null until the app has said. */
  model: ToolPanelModel | null;
  /** How far into its sound it starts, in ms. */
  offsetMs: number;
  onChange: (change: AudioClipChange) => void;
  onDownloadModel: () => void;
  /** Hearing the clip from where it starts, in the island. */
  preview: SoundPreview;
  /** How far its singing is separated, or null while none is under way. */
  separating: number | null;
  /** How long its sound is, in ms. */
  soundMs: number;
  /** Its sound's title, or the name of the file it came from. */
  title: string;
  /** How much louder or quieter than its default level, in dB. */
  volumeDb: number;
};

export function audioClipIsland(
  props: AudioClipIslandProps,
): EditorIslandContent {
  const { onChange } = props;
  const Glyph = props.isSong ? Music : AudioLines;
  return {
    actions: (
      <ConfirmActionButton
        confirmLabel={t("editor-island-audio-confirm-delete")}
        icon={<Trash2 aria-hidden />}
        label={t("editor-island-audio-delete")}
        onConfirm={() => {
          onChange({ delete: true });
        }}
        variant="ghost"
      />
    ),
    anchor: "end",
    key: `audio-clip-${props.id}`,
    section: (
      <div className="flex flex-col gap-separate">
        <SoundStartPicker
          clipMs={props.clipMs}
          // A looped clip may start anywhere; one that plays once has to
          // fit in what is left of its sound.
          latestMs={Math.max(
            0,
            props.soundMs - (props.isLooped ? LEAST_CLIP_MS : props.clipMs),
          )}
          levels={props.levels}
          offsetMs={props.offsetMs}
          onChange={(offsetMs) => {
            onChange({ offsetMs });
          }}
          preview={props.preview}
          soundMs={props.soundMs}
        />
        <AudioClipVolumeRow
          onChange={(volumeDb) => {
            onChange({ volumeDb });
          }}
          volumeDb={props.volumeDb}
        />
        <AnnotationOptions>
          <LoopToggle
            key="loop"
            loops={props.isLooped}
            onChange={(isLooped) => {
              onChange({ isLooped });
            }}
          />
          {props.isSong ? (
            <InstrumentalToggle
              isInstrumental={props.isInstrumental}
              key="instrumental"
              onChange={(isInstrumental) => {
                onChange({ isInstrumental });
              }}
            />
          ) : null}
        </AnnotationOptions>
        <InstrumentalNote {...props} />
        <div className="flex justify-end">
          <Button
            onPress={() => {
              onChange({ replace: true });
            }}
          >
            {t("editor-island-audio-replace")}
          </Button>
        </div>
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<Glyph aria-hidden className="size-icon" />}
        title={<span dir="auto">{props.title}</span>}
      />
    ),
  };
}

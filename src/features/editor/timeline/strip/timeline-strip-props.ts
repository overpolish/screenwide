// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { type TimelineCaption } from "../caption-lane/caption-line";
import { type TimelineScene } from "../scene-rail/scene-clip";
import { type Playhead } from "../scrub-playhead";
import { type TimelineShortcut } from "../shortcut-lane/shortcut-chip";
import { type Span, type TimeMap } from "../timeline-scale";

import {
  type StripBeat,
  type StripBeatRowProps,
  type StripLull,
} from "./strip-beat-row";
import { type StripSelection } from "./strip-selection";

/** What the strip draws and what it tells of every change made on it. */
export type TimelineStripProps = Pick<
  StripBeatRowProps,
  | "cuts"
  | "isAltHeld"
  | "onBeatTrim"
  | "onLullPlayChange"
  | "onLullTrim"
  | "onRestore"
  | "onSeek"
  | "pictureAspect"
  | "selection"
> & {
  annotations: TimelineAnnotation[];
  beats: StripBeat[];
  /** The voice's lines, or null where the recording has no voice. */
  captions: TimelineCaption[] | null;
  /** Whether the captions are hidden from the picture. */
  isCaptionsHidden: boolean;
  /** Whether the scissors are in hand, from the Cut tool: a drag across
   * the strip draws a stretch to cut, as with ⌥ held, and a press splits. */
  isCutting: boolean;
  /** Whether the recording plays, which the view follows. */
  isPlaying: boolean;
  /** The recorded moments where play jumps, each the start of a stretch
   * that does not follow what plays before it. */
  jumps: number[];
  lulls: StripLull[];
  /** How the recording plays now. */
  map: TimeMap;
  /** The most the lulls can be squeezed by, in seconds. */
  maxSqueeze: number;
  onAnnotationRestack: (lanes: Record<string, number>) => void;
  onAnnotationRetime: (id: string, start: number, end: number) => void;
  onCaptionPress: (line: TimelineCaption) => void;
  /** Told shortcuts to delete, by id. */
  onDeleteShortcuts: (ids: string[]) => void;
  /** Told stretches moved, in the order they play, to play together
   * before a recorded moment, or at the end for null. */
  onMove: (spans: Span[], before: number | null) => void;
  /** Told a jump pressed, by the recorded moment it names, to put what was
   * moved there back where it was recorded. */
  onPutBack: (at: number) => void;
  /** Told a split pressed, by the recorded moment it is at, to take it away.
   */
  onRemoveSplit: (at: number) => void;
  onSceneDraw: (from: number, to: number) => void;
  onSceneRetime: (id: string, start: number, end: number) => void;
  /** Told what is chosen, or null when the choice is let go. */
  onSelect: (selection: StripSelection | null) => void;
  /** Told the scissors pressed without drawing, to split the recording at
   * a recorded moment, where something moved can then land. */
  onSplit: (at: number) => void;
  onSqueezeChange: (squeeze: number) => void;
  /** Where the playhead is, in the edited recording. */
  playhead: Playhead;
  /** How long the recording was recorded, in seconds. */
  recorded: number;
  scenes: TimelineScene[];
  /** The shortcuts pressed while recording, where they show. */
  shortcuts: TimelineShortcut[];
  /** The recorded moments the recording is split at. */
  splits: number[];
  /** How many seconds the lulls are squeezed by now. */
  squeeze: number;
};

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  SceneChoice,
  SceneSpanTarget,
} from "../../recording/scenes/recording-scene-span";
import { SceneOffer } from "../scene/scene-offer";
import {
  ToolPanelTranscript,
  TranscriptCorrection,
  TranscriptMove,
  TranscriptRange,
} from "../tool-panel-transcript";

import { TranscriptWord } from "./transcript-words";

export type TranscriptPanelViewProps = {
  isLocked: boolean;
  /** The language the transcriber heard, as its code; null until read. */
  language: string | null;
  onCancel: () => void;
  /** Give the words this stretch covers this text. */
  onCorrect: (correction: TranscriptCorrection) => void;
  /** Apply every correction in one undo step, for Replace all. */
  onCorrectMany: (corrections: TranscriptCorrection[]) => void;
  /** Cut these stretches, each one run of words still kept. */
  onCut: (ranges: TranscriptRange[]) => void;
  onDownload: () => void;
  /** Show this stretch on the editor's timeline as the words chosen, or
   * none with null. */
  onHighlight: (range: TranscriptRange | null) => void;
  /** Move the footage the dragged words chose, in one undo step. */
  onMove: (move: TranscriptMove) => void;
  /** Stop marking `text`, folded, as profanity anywhere in the recording. */
  onNotProfanity: (text: string) => void;
  onRedo: () => void;
  /** Run Remove silences, the microphone's own. */
  onRemoveSilences: () => void;
  /** Bring back these stretches, each one run of struck words. */
  onRestore: (ranges: TranscriptRange[]) => void;
  /** Bring back everything the panel cut, and every silence Remove
   * silences cut. */
  onRestoreAll: () => void;
  /** Add the scene `target` names, or lay out the one it names as `choice`
   * and move it to the words: a pick in the words' menu. */
  onScene: (target: SceneSpanTarget, choice: SceneChoice) => void;
  onSeek: (ms: number) => void;
  onTogglePlayback: () => void;
  onTranscribe: () => void;
  onUndo: () => void;
  /** What Remove silences would cut, so hovering Remove can show which
   * pauses it acts on. */
  plannedSilences: readonly TranscriptRange[];
  playheadMs: number;
  /** What a scene for the words can be laid out as; null where the
   * recording has no screen to arrange, or the editor is locked, and the
   * words' menu offers no scenes. */
  sceneOffer: SceneOffer | null;
  /** Null until the editor has published one. */
  transcript: ToolPanelTranscript | null;
  /** Null until the transcript has been read. */
  words: readonly TranscriptWord[] | null;
};

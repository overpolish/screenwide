// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Captions } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { type AnnotationColorProps } from "../annotation/annotation-island-parts";
import { WordByWordToggle } from "../annotation/text/text-toggles";
import { IslandRowPresence } from "../island-row-presence";
import { ShowGlyph } from "../keyboard/keyboard-glyphs";

import { CaptionsSection } from "./captions-section";
import { CaptionsTranscriptLine } from "./captions-transcript-line";

import type { CaptionSettings } from "../../../../bindings/CaptionSettings";
import type { FontFamily } from "../../../../bindings/FontFamily";
import type { TextLayout } from "../../../../bindings/TextLayout";
import type { TextStyle } from "../../../../bindings/TextStyle";
import type { TextTemplates } from "../../recording/captions/use-text-templates";
import type { ToolPanelModel } from "../../tool-panels/tool-panel-selection";
import type { EditorIslandContent } from "../editor-island";

// The recording's captions, all of them at once, as a text placed by hand
// is dressed: the same template, faces and colours, laid out the same way,
// with the captions' own choices among them. Its row ends with whether
// their words come on one after another, as a text's does, and whether
// they show at all; hidden, there is nothing else to set, so the section
// folds away, as the Keyboard island's does. Under the row: the template;
// how large the type is from the template's size and whether it is set in
// capitals, the key word's size where the template has one, where on the
// picture the captions sit and how many lines they take, and the most
// words one holds; the faces; what the captions leave out and how
// profanity shows; the colours; then bringing back the captions taken out
// of the video, putting the look back as last set as default, and setting
// it as the default for new captions. Captions are made of the
// transcript's words, so until there is one the section says how it is
// coming, or offers the model it needs.

/** Where the transcript the captions are made of stands. */
export type CaptionsTranscript =
  | { model: ToolPanelModel; status: "noModel" }
  | { progress: number; status: "transcribing" }
  | { status: "failed" }
  | { status: "noSpeech" }
  | { status: "ready" };

export type CaptionsIslandProps = AnnotationColorProps & {
  /** Whether Reset has anything to put back. */
  canReset: boolean;
  /** Whether the look differs from new captions' default. */
  canSetDefault: boolean;
  /** Whether the captions are drawn over the recording. */
  enabled: boolean;
  /** How many captions have been taken out of the video, to bring back. */
  hiddenCount: number;
  /** Their look, the layout options among it. */
  look: TextStyle;
  onDownloadModel: () => void;
  onEnabledChange: (enabled: boolean) => void;
  /** A layout option set by hand, which another template keeps. */
  onLayoutChange: (layout: TextLayout) => void;
  /** A change to the look, a template chosen among them, which brings its
   * own look but for the layout options set by hand. */
  onLookChange: (patch: Partial<TextStyle>) => void;
  /** Puts the look, the layout and the rules back as last set as
   * default. */
  onReset: () => void;
  /** Brings back every caption taken out of the video. */
  onRestoreHidden: () => void;
  /** Makes new captions start looking and laid out as these, with these
   * rules. */
  onSetDefault: () => void;
  onSettingsChange: (settings: Partial<CaptionSettings>) => void;
  /** What the captions say and when, whatever their look. */
  settings: CaptionSettings;
  templates: TextTemplates;
  transcript: CaptionsTranscript;
  /** Families beyond the system's, for a story. */
  families?: FontFamily[];
};

export function captionsIsland(
  props: CaptionsIslandProps,
): EditorIslandContent {
  const { transcript } = props;
  const isReady = transcript.status === "ready";
  return {
    actions: isReady ? (
      <>
        <IslandRowPresence isShown={props.enabled}>
          <WordByWordToggle
            onChange={(revealWords) => {
              props.onLookChange({ revealWords });
            }}
            revealWords={props.look.revealWords}
          />
        </IslandRowPresence>
        <OptionToggle
          glyph={(isOn) => <ShowGlyph isShown={isOn} />}
          isOn={props.enabled}
          label={t("editor-island-captions-show")}
          onChange={props.onEnabledChange}
        />
      </>
    ) : undefined,
    // Hung from the row's start, so the pickers' column opens past its end.
    anchor: "start",
    key: "captions",
    section:
      transcript.status !== "ready" ? (
        <CaptionsTranscriptLine
          onDownloadModel={props.onDownloadModel}
          transcript={transcript}
        />
      ) : props.enabled ? (
        <CaptionsSection {...props} />
      ) : undefined,
    sectionKey:
      transcript.status === "noModel" &&
      transcript.model.status === "downloading"
        ? "downloading"
        : transcript.status,
    title: (
      <IslandRowTitle
        glyph={<Captions aria-hidden className="size-icon" />}
        title={t("editor-island-captions")}
      />
    ),
  };
}

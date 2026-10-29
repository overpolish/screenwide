// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScrollText } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { islandTranscriptKey } from "../island-transcript";
import { IslandTranscriptLine } from "../island-transcript-line";

import { ScriptCaptions } from "./script-captions";
import { ScriptSearchField } from "./script-search-field";
import { ScriptTranscript } from "./script-transcript";

import type { EditorIslandContent } from "../editor-island";
import type { ScriptIslandProps } from "./script-island-props";

// The recording's script: what was said, as a whole, to read and put right
// in one pass rather than a caption at a time. It stands at the editor's
// end, the picture between it and any island it is open beside. With the
// Captions island, it shows the script as the captions break it, and edits
// their breaks, hidden words and key words. The Transcript mode opens it on
// its own, to cut: words removed, restored and moved, pauses, fillers and
// profanity cleared at once, and a search at the row's end that finds and
// replaces. A word's spelling is the transcript's, so a correction made in
// either shows wherever the script does.
const scriptTitle = () => (
  <IslandRowTitle
    glyph={<ScrollText aria-hidden className="size-icon" />}
    title={t("editor-island-script")}
  />
);

export function scriptIsland(props: ScriptIslandProps): EditorIslandContent {
  if (props.mode === "captions")
    return {
      anchor: "end",
      key: "script",
      section: <ScriptCaptions {...props} />,
      title: scriptTitle(),
    };
  const { transcript } = props;
  const isReady = transcript.status === "ready";
  return {
    actions: isReady ? (
      <ScriptSearchField isDisabled={props.isLocked} />
    ) : undefined,
    anchor: "end",
    key: "script",
    section: isReady ? (
      <ScriptTranscript {...props} />
    ) : (
      <IslandTranscriptLine
        onDownloadModel={props.onDownloadModel}
        transcript={transcript}
      />
    ),
    sectionKey: islandTranscriptKey(transcript),
    title: scriptTitle(),
  };
}

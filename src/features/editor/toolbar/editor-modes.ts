// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Captions,
  Keyboard,
  LayoutPanelLeft,
  type LucideIcon,
  Monitor,
  MousePointerClick,
  ScrollText,
  Video,
  Volume2,
} from "lucide-react";

import { t } from "../../../i18n/i18n";
import { FrameToolIcon } from "../glyphs/frame-tool-icon";

import type { AudioTrackKind } from "../../../bindings/AudioTrackKind";
import type { RecordingVideoTrackId } from "../types";

// The modes the title bar opens (`EditorModeBar`), each named and drawn as
// the island it opens is titled, so the button and the island read as one.
// A mode works on the whole recording rather than on something chosen in it.

export type EditorKind = "audio" | "recording" | "screenshot";

export type EditorMode =
  | "camera"
  | "captions"
  | "cursor"
  | "frame"
  | "keyboard"
  | "scene"
  | "screen"
  | "sound"
  | "transcript";

type EditorModeButton = {
  icon: LucideIcon;
  id: EditorMode;
  label: string;
  /** The key that opens it, in the hotkey format tooltips read
   * (`Shift+KeyC`), as the editor's key map has it. */
  shortcut?: string;
};

/** The key that opens each mode. The editor's key map
 * (`editorToolKeyAction`) answers them, and the Shortcuts pane lists them. */
export const MODE_SHORTCUTS = {
  camera: "Shift+KeyM",
  captions: "U",
  cursor: "Shift+KeyC",
  frame: "F",
  keyboard: "Shift+KeyK",
  scene: "L",
  screen: "Shift+KeyS",
  sound: "Shift+KeyU",
  transcript: "Shift+KeyT",
} satisfies Record<EditorMode, string>;

/** The key that brings out the Tracks island beside the modes. */
export const TRACKS_SHORTCUT = "Shift+KeyA";

/** The key that brings out the Library island beside Tracks. */
export const LIBRARY_SHORTCUT = "Shift+KeyL";

/** A group of modes the title bar sets apart, named for what it is of: the
 * picture, what was done while recording, and what was said and heard. */
export type EditorModeGroup = {
  id: "done" | "heard" | "picture";
  modes: EditorModeButton[];
};

/** What a recording was made with, which never changes while it is open:
 * its own picture (its screen, or the camera a recording of the camera
 * alone is of; none for audio), a camera beside it, the cursor's movement
 * and the keys pressed, and its sound, each stream told by its kind. */
export type RecordedParts = {
  audio: readonly { kind: AudioTrackKind; streamIndex: number }[];
  camera: boolean;
  cursor: boolean;
  keyboard: boolean;
  picture: CapturePicture | null;
};

type CapturePicture = "camera" | "screen";

/** Where the microphone's transcript stands: still to be made (no model
 * yet, transcribing, failed, or not begun), or made, with words heard or
 * none. */
export type TranscriptState = "heard" | "pending" | "silent";

/** What a capture holds in its export as it stands, each part offering the
 * modes made of it. */
export type CaptureParts = {
  /** Any sound. */
  audio: boolean;
  /** A camera beside the recording's own picture. */
  camera: boolean;
  /** The cursor's movement, recorded. */
  cursor: boolean;
  /** The keys pressed, recorded. */
  keyboard: boolean;
  /** A microphone recorded, which the transcript is made of, its track in
   * the export or not. */
  microphone: boolean;
  /** The recording's own picture, or null with none: an audio recording,
   * or the picture left out of the export. */
  picture: CapturePicture | null;
  transcript: TranscriptState;
};

/** A screenshot's: it has none. */
export const NO_CAPTURE_PARTS: CaptureParts = {
  audio: false,
  camera: false,
  cursor: false,
  keyboard: false,
  microphone: false,
  picture: null,
  transcript: "pending",
};

/** A recording made with nothing, until the editor says what it was. */
export const NOTHING_RECORDED: RecordedParts = {
  audio: [],
  camera: false,
  cursor: false,
  keyboard: false,
  picture: null,
};

/** What a recording holds in its export now: what it was made with, less
 * the tracks left out of it (the Tracks island's switches). */
export function captureParts({
  enabledStreams,
  enabledVideoTracks,
  recorded,
  transcript,
}: {
  /** The audio streams in the export. */
  enabledStreams: ReadonlySet<number>;
  /** The video tracks in the export. */
  enabledVideoTracks: ReadonlySet<RecordingVideoTrackId>;
  recorded: RecordedParts;
  transcript: TranscriptState;
}): CaptureParts {
  const audio = recorded.audio.filter(({ streamIndex }) =>
    enabledStreams.has(streamIndex),
  );
  return {
    audio: audio.length > 0,
    camera: recorded.camera && enabledVideoTracks.has("camera"),
    cursor: recorded.cursor,
    keyboard: recorded.keyboard,
    // What was said is captioned and transcribed whether or not its sound
    // is in the export.
    microphone: recorded.audio.some(({ kind }) => kind === "microphone"),
    picture: enabledVideoTracks.has("primary") ? recorded.picture : null,
    transcript,
  };
}

/** How many tracks the Tracks island has to switch: the recording's
 * picture, a camera beside it, and each audio stream. */
export const recordedTrackCount = (recorded: RecordedParts) =>
  (recorded.picture ? 1 : 0) +
  (recorded.camera ? 1 : 0) +
  recorded.audio.length;

/** The modes offered for a capture as its export stands, in groups that the
 * title bar divides: the picture (its frame, the screen, the camera, the
 * scene), what was done while recording (the cursor, the keys), then what
 * was said and heard. A mode is offered only while the export holds
 * something for it to work on (DECISIONS.md, "Which modes are offered").
 * A screenshot always has its layers, so it always offers its Frame and
 * Screenshot. */
export function editorModeGroups({
  kind,
  parts,
}: {
  kind: EditorKind;
  parts: CaptureParts;
}): EditorModeGroup[] {
  const frame: EditorModeButton = {
    icon: FrameToolIcon,
    id: "frame",
    label: t("editor-tool-frame"),
    shortcut: MODE_SHORTCUTS.frame,
  };
  const screen: EditorModeButton = {
    icon: Monitor,
    id: "screen",
    label:
      kind === "screenshot"
        ? t("editor-island-screenshot")
        : t("editor-island-screen"),
    shortcut: MODE_SHORTCUTS.screen,
  };
  if (kind === "screenshot") return [{ id: "picture", modes: [frame, screen] }];
  const hasPicture = parts.picture !== null || parts.camera;
  // Until the transcript is made, Captions and Transcript are where its
  // model is fetched and its progress shown; made with nothing heard, they
  // have nothing to work on.
  const hasSpeech = parts.microphone && parts.transcript !== "silent";
  // The cursor and the keys are drawn over the screen.
  const hasScreen = parts.picture === "screen";
  // A recording of the camera alone has its camera as its own picture,
  // which the Camera mode works on with the camera's own tools; it has no
  // screen, so no Screen.
  const camera: EditorModeButton = {
    icon: Video,
    id: "camera",
    label: t("editor-island-camera"),
    shortcut: MODE_SHORTCUTS.camera,
  };
  const groups: {
    id: EditorModeGroup["id"];
    modes: (EditorModeButton | false)[];
  }[] = [
    {
      id: "picture",
      modes: [
        hasPicture && frame,
        hasScreen && screen,
        (parts.camera || parts.picture === "camera") && camera,
        // A scene frames the picture, alone too: it zooms and moves it, and
        // the zooms planned from the cursor and keys are scenes.
        parts.picture !== null && {
          icon: LayoutPanelLeft,
          id: "scene",
          label: t("editor-island-scene"),
          shortcut: MODE_SHORTCUTS.scene,
        },
      ],
    },
    {
      id: "done",
      modes: [
        hasScreen &&
          parts.cursor && {
            icon: MousePointerClick,
            id: "cursor",
            label: t("editor-island-cursor"),
            shortcut: MODE_SHORTCUTS.cursor,
          },
        hasScreen &&
          parts.keyboard && {
            icon: Keyboard,
            id: "keyboard",
            label: t("editor-island-keyboard"),
            shortcut: MODE_SHORTCUTS.keyboard,
          },
      ],
    },
    {
      id: "heard",
      modes: [
        // Captions are drawn over the picture.
        hasPicture &&
          hasSpeech && {
            icon: Captions,
            id: "captions",
            label: t("editor-island-captions"),
            shortcut: MODE_SHORTCUTS.captions,
          },
        hasSpeech && {
          icon: ScrollText,
          id: "transcript",
          label: t("editor-toolbar-transcript"),
          shortcut: MODE_SHORTCUTS.transcript,
        },
        parts.audio && {
          icon: Volume2,
          id: "sound",
          label: t("editor-island-sound"),
          shortcut: MODE_SHORTCUTS.sound,
        },
      ],
    },
  ];
  // A group left with nothing in it takes its gap with it.
  return groups
    .map(({ id, modes }) => ({
      id,
      modes: modes.filter((mode) => mode !== false),
    }))
    .filter(({ modes }) => modes.length > 0);
}

/** Every mode in `groups`, which the keys that open them ask. */
export const offeredModes = (groups: EditorModeGroup[]) =>
  new Set(groups.flatMap(({ modes }) => modes.map(({ id }) => id)));

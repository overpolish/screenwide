// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useNumberFormatter } from "react-aria";
import { Button } from "react-aria-components";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { formatDuration } from "../../duration";
import { removeSound, type Sound } from "../../sound-track/sound-library";
import { Waveform } from "../../waveform/waveform";
import { IslandLineButton } from "../island-line-button";
import { IslandPlayButton } from "../island-play-button";

import { LibraryNameField } from "./library-name-field";
import { useDragOut } from "./use-drag-out";

// One sound in the library: play, as a moment's voice note plays; its
// title, a line pressed to add it, double-clicked (or F2) to rename it, and
// dragged out where the system lets a drag start; its waveform, pressed
// anywhere to hear it from there; its length, as the transport shows a
// length; and its bin, asking twice, which takes it out of the library,
// where projects that play it keep their own copy. A click adds once the
// time for a second click has passed, so a double-click only renames.

/** How long after a click it is added, in ms, unless a second click makes
 * it a double-click. */
const DOUBLE_CLICK_MS = 300;

/** Below this, in ms, a length is shown in seconds to a tenth. */
const SHORT_MS = 10_000;

export function LibraryRow({
  isPlaying,
  onAdd,
  onDrag,
  onPlayFrom,
  onRename,
  onTogglePlay,
  played,
  sound,
}: {
  isPlaying: boolean;
  onAdd: () => void;
  /** Starts dragging its file out, from the press under way. */
  onDrag: () => void;
  /** Hears it from a moment of it, in ms. */
  onPlayFrom: (fromMs: number) => void;
  onRename: (title: string) => void;
  onTogglePlay: () => void;
  /** How far into it it has played, from 0 to 1. */
  played: number;
  sound: Sound;
}) {
  const dragOut = useDragOut(onDrag);
  const tenths = useNumberFormatter({
    maximumFractionDigits: 1,
    minimumFractionDigits: 1,
  });
  const [isEditing, setIsEditing] = useState(false);
  const addingRef = useRef(0);
  useEffect(
    () => () => {
      window.clearTimeout(addingRef.current);
    },
    [],
  );
  const edit = () => {
    window.clearTimeout(addingRef.current);
    addingRef.current = 0;
    setIsEditing(true);
  };
  // A second press while the first still waits to add is a double-click.
  const press = (isKeyboard: boolean) => {
    if (isKeyboard) {
      onAdd();
      return;
    }
    if (addingRef.current !== 0) {
      edit();
      return;
    }
    addingRef.current = window.setTimeout(() => {
      addingRef.current = 0;
      onAdd();
    }, DOUBLE_CLICK_MS);
  };
  // Under ten seconds, as sound effects mostly are, whole seconds tell
  // little apart and a short one would read 00:00.
  const length =
    sound.durationMs < SHORT_MS
      ? t("editor-island-sound-library-seconds", {
          seconds: tenths.format(sound.durationMs / 1_000),
        })
      : formatDuration(sound.durationMs);
  return (
    <li className="flex items-center gap-inline">
      <IslandPlayButton
        isPlaying={isPlaying}
        label={
          isPlaying
            ? t("editor-island-sound-library-stop", { title: sound.title })
            : t("editor-island-sound-library-play", { title: sound.title })
        }
        onPress={onTogglePlay}
      />
      {isEditing ? (
        <LibraryNameField
          className="px-inset-sm text-label text-fg"
          label={t("editor-island-library-name")}
          onDone={() => {
            setIsEditing(false);
          }}
          onRename={onRename}
          title={sound.title}
        />
      ) : (
        <span className="flex min-w-0 grow" {...dragOut}>
          <IslandLineButton
            aria-label={t("editor-island-sound-library-add", {
              length,
              title: sound.title,
            })}
            className="min-w-0"
            onKeyDown={(event) => {
              if (event.key === "F2") edit();
              else event.continuePropagation();
            }}
            onPress={(event) => {
              press(event.pointerType === "keyboard");
            }}
          >
            <span className="min-w-0 truncate" dir="auto">
              {sound.title}
            </span>
          </IslandLineButton>
        </span>
      )}
      <Button
        aria-label={t("editor-island-sound-library-play-from", {
          title: sound.title,
        })}
        className={cn(
          "h-island-control w-28 shrink-0 cursor-pointer rounded-full",
          focusStyles,
          elementFocusVisible,
        )}
        onPress={(event) => {
          // A key plays it from its start; the pointer, from where it is.
          const width = event.target.getBoundingClientRect().width;
          onPlayFrom(
            event.pointerType === "keyboard"
              ? 0
              : (event.x / Math.max(width, 1)) * sound.durationMs,
          );
        }}
      >
        <Waveform
          aria-label={sound.title}
          bars={36}
          className="h-5 w-full"
          color="var(--color-fg)"
          peaks={sound.levels}
          progress={played}
        />
      </Button>
      <span className="shrink-0 text-caption whitespace-nowrap text-fg-tertiary tabular-nums">
        {length}
      </span>
      <ConfirmActionButton
        className="shrink-0"
        confirmLabel={t("editor-island-library-confirm-remove")}
        icon={<Trash2 aria-hidden />}
        label={t("editor-island-sound-remove", { title: sound.title })}
        onConfirm={() => {
          removeSound(sound.name).catch((cause: unknown) => {
            console.error("Could not remove the sound", cause);
          });
        }}
        variant="ghost"
      />
    </li>
  );
}

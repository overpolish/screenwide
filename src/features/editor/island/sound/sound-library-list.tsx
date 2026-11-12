// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";
import { useState } from "react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";
import { Text } from "../../../../primitives/text/text";
import { formatDuration } from "../../duration";
import {
  removeSound,
  type Sound,
  useSoundLibrary,
} from "../../sound-track/sound-library";

// The library of songs and sound effects, under the recording's tracks on
// the Sound island: every sound added to any recording, the most recent
// first. One chosen there is added to the timeline, or taken out of the
// library, asking twice; projects that play it keep their own copy. More
// are added from a file. Where each lands is the lanes' rule.

/** What the island asks of the editor: a sound from the library, or ones
 * chosen from a file. */
export type SoundAddition = { name: string } | { fromDisk: true };

/** How many sounds show before the list scrolls. */
const SHOWN = 6;

function SoundRow({ sound }: { sound: Sound }) {
  return (
    <ListBoxItem id={sound.name} textValue={sound.title}>
      <span className="flex min-w-0 items-center gap-separate">
        <span className="min-w-0 grow truncate" dir="auto">
          {sound.title}
        </span>
        <Text as="span" className="shrink-0 font-mono" variant="caption">
          {formatDuration(sound.durationMs)}
        </Text>
      </span>
    </ListBoxItem>
  );
}

export function SoundLibraryList({
  onAdd,
}: {
  onAdd: (addition: SoundAddition) => void;
}) {
  const library = useSoundLibrary();
  const [chosen, setChosen] = useState<string | null>(null);
  const isChosen =
    chosen !== null && library.some(({ name }) => name === chosen);
  return (
    <div className="flex flex-col gap-inline">
      <Text as="h3" className="px-inset-sm" variant="label">
        {t("editor-island-sound-library")}
      </Text>
      {library.length === 0 ? (
        <Text className="px-inset-sm text-fg-secondary" variant="caption">
          {t("editor-island-sound-library-empty")}
        </Text>
      ) : (
        <ListBox
          aria-label={t("editor-island-sound-library")}
          items={library}
          onAction={(name) => {
            onAdd({ name: String(name) });
          }}
          onSelectionChange={(keys) => {
            setChosen(
              keys === "all" || keys.size === 0 ? null : String([...keys][0]),
            );
          }}
          selectedKeys={isChosen ? [chosen] : []}
          // A click chooses a sound, so it can be added or removed with the
          // buttons under the list; a double click or Return adds it.
          selectionBehavior="replace"
          selectionMode="single"
          style={{
            maxHeight: `calc(var(--spacing-control-height) * ${String(SHOWN)})`,
          }}
        >
          {(sound) => <SoundRow sound={sound} />}
        </ListBox>
      )}
      <div className="flex justify-end gap-separate">
        {isChosen ? (
          <ConfirmActionButton
            confirmLabel={t("editor-island-sound-confirm-remove")}
            icon={<Trash2 aria-hidden />}
            label={t("editor-island-sound-remove")}
            onConfirm={() => {
              removeSound(chosen).catch((cause: unknown) => {
                console.error("Could not remove the sound", cause);
              });
              setChosen(null);
            }}
            variant="ghost"
          />
        ) : null}
        <Button
          onPress={() => {
            onAdd({ fromDisk: true });
          }}
        >
          {t("editor-island-sound-add-from-file")}
        </Button>
        <Button
          color="primary"
          isDisabled={!isChosen}
          onPress={() => {
            if (isChosen) onAdd({ name: chosen });
          }}
        >
          {t("editor-island-sound-add")}
        </Button>
      </div>
    </div>
  );
}

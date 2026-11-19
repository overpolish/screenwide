// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { CueCard } from "./cue-card";
import {
  CUE_SHOW_EVENT,
  hideTeleprompterCue,
  type TeleprompterCuePicture,
  teleprompterCue,
} from "./cue-window-api";

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

// The window beside the teleprompter (`src-tauri/src/teleprompter/
// cue_window.rs`), sized to the cue's picture, which fills it. It asks for
// the cue as it loads, since one sent before then is lost, and takes itself
// off screen once the cue has faded out.
export function TeleprompterCueWindow() {
  const [picture, setPicture] = useState<TeleprompterCuePicture | null>(null);

  useEffect(() => {
    teleprompterCue().then(setPicture).catch(report("Could not read the cue"));
    const unlistening = listen<TeleprompterCuePicture | null>(
      CUE_SHOW_EVENT,
      ({ payload }) => {
        setPicture(payload);
      },
    );
    return () => {
      void unlistening.then((unlisten) => {
        unlisten();
      });
    };
  }, []);

  return (
    <div
      className="size-full"
      style={{ transform: picture?.mirrored ? "scaleX(-1)" : undefined }}
    >
      <CueCard
        cue={
          picture && {
            id: picture.path,
            src: convertFileSrc(picture.path),
            title: picture.title,
          }
        }
        onGone={() => {
          hideTeleprompterCue().catch(report("Could not hide the cue"));
        }}
        place={{ kind: "window" }}
      />
    </div>
  );
}

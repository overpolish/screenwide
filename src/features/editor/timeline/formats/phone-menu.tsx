// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown, Smartphone } from "lucide-react";
import { useRef } from "react";

import { t } from "../../../../i18n/i18n";
import {
  IconButton,
  IconToggleButton,
} from "../../../../primitives/button/icon-button";
import {
  boundsAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";
import { PHONES, phoneName } from "../../recording/formats/phones";

import type { Phone } from "../../../../bindings/Phone";
import type { FrameFormatPatch } from "../../recording/formats/use-recording-formats";
import type { ToolPanelFormats } from "../../tools/tool-panel-frame";

const MENU = "format-phone:";
const MENU_WIDTH = 240;
const NONE = "none";

/**
 * What platforms watched in a phone app (Shorts, TikTok, Reels) are
 * previewed on, at the end of the format bar, in two parts: the phone's
 * glyph turns the preview (the phone, and the interface of the app the
 * platform in hand is made for) on and off, off until asked; the chevron
 * beside it picks the phone, or none, for every such platform. Always
 * there, it waits while the canvas in hand has no app to preview. The video
 * fills the phone's screen, as the apps show a 9:16 video.
 */
export function PhoneMenu({
  formats,
  isDisabled,
  onPatch,
}: {
  formats: ToolPanelFormats;
  isDisabled: boolean;
  onPatch: (patch: FrameFormatPatch) => void;
}) {
  const triggerRef = useRef<HTMLButtonElement>(null);
  const open = usePopupMenu({
    idPrefix: MENU,
    label: t("editor-frame-phone"),
    mode: "select",
    onSelect: (id) => {
      onPatch({
        kind: "phone",
        phone: id === NONE ? null : { phone: id as Phone },
      });
    },
    // Beside the format bar at the window's foot, with the room above it.
    opensAbove: true,
    width: MENU_WIDTH,
  });
  const phoneLabel = formats.phone
    ? phoneName(formats.phone.phone)
    : t("editor-frame-phone-none");
  return (
    <div className="flex shrink-0 items-center gap-px">
      <IconToggleButton
        aria-label={t("editor-frame-phone-preview")}
        className="rounded-e-none"
        isDisabled={isDisabled}
        isSelected={formats.preview && !isDisabled}
        onChange={(isShown) => {
          onPatch({ isShown, kind: "preview" });
        }}
        variant="solid"
      >
        <Smartphone aria-hidden />
      </IconToggleButton>
      <IconButton
        aria-label={t("editor-frame-phone-current", { phone: phoneLabel })}
        className="rounded-s-none"
        isDisabled={isDisabled}
        onPress={() => {
          const bounds = triggerRef.current?.getBoundingClientRect();
          if (!bounds) return;
          void open({
            anchor: boundsAnchor(bounds),
            items: [
              { id: NONE, label: t("editor-frame-phone-none") },
              ...PHONES.map((phone) => ({
                id: phone,
                label: phoneName(phone),
              })),
            ],
            selectedIds: [formats.phone?.phone ?? NONE],
          });
        }}
        ref={triggerRef}
        variant="solid"
      >
        <ChevronDown aria-hidden />
      </IconButton>
    </div>
  );
}

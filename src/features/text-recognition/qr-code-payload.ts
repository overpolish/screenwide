// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../i18n/i18n";

type ActionPayload = {
  action: "email" | "link" | "phone" | "sms";
  kind: "action";
  label: string;
  url: string;
};

type InformationPayload = {
  kind: "information";
  label: string;
};

type UnsupportedPayload = {
  kind: "unsupported";
  label: string;
  reason: string;
};

export type QrPayload = ActionPayload | InformationPayload | UnsupportedPayload;

const unsupported = (reason: string): UnsupportedPayload => ({
  kind: "unsupported",
  label: t("text-recognition-qr-unsupported"),
  reason,
});

const information = (label: string): InformationPayload => ({
  kind: "information",
  label,
});

const structuredPayload = (content: string): QrPayload | undefined => {
  if (/^WIFI:/iu.test(content)) {
    const ssid = /(?:^|;)S:((?:\\.|[^;])*)/iu.exec(content.slice(5))?.[1];
    return ssid
      ? information(t("text-recognition-qr-wifi"))
      : unsupported(t("text-recognition-qr-wifi-incomplete"));
  }
  if (/^BEGIN:VCARD\b/iu.test(content))
    return /END:VCARD\s*$/iu.test(content)
      ? information(t("text-recognition-qr-contact"))
      : unsupported(t("text-recognition-qr-contact-incomplete"));
  if (/^BEGIN:(?:VCALENDAR|VEVENT)\b/iu.test(content))
    return /END:(?:VCALENDAR|VEVENT)\s*$/iu.test(content)
      ? information(t("text-recognition-qr-calendar"))
      : unsupported(t("text-recognition-qr-calendar-incomplete"));
};

export const classifyQrPayload = (
  rawContent: string,
  decodeError?: string,
): QrPayload => {
  if (decodeError) return unsupported(decodeError);
  const content = rawContent.trim();
  if (!content) return unsupported(t("text-recognition-qr-no-content"));
  const structured = structuredPayload(content);
  if (structured) return structured;
  if (!/^[a-z][a-z\d+.-]*:/iu.test(content))
    return information(t("text-recognition-qr-text"));

  let url: URL;
  try {
    url = new URL(content);
  } catch {
    return unsupported(t("text-recognition-qr-malformed"));
  }

  switch (url.protocol) {
    case "http:":
    case "https:":
      return url.hostname
        ? {
            action: "link",
            kind: "action",
            label: t("text-recognition-qr-open-link"),
            url: url.toString(),
          }
        : unsupported(t("text-recognition-qr-link-incomplete"));
    case "tel:":
      return url.pathname
        ? {
            action: "phone",
            kind: "action",
            label: t("text-recognition-qr-call"),
            url: url.toString(),
          }
        : unsupported(t("text-recognition-qr-phone-incomplete"));
    case "mailto:":
      return url.pathname
        ? {
            action: "email",
            kind: "action",
            label: t("text-recognition-qr-email"),
            url: url.toString(),
          }
        : unsupported(t("text-recognition-qr-email-incomplete"));
    case "sms:":
      return url.pathname
        ? {
            action: "sms",
            kind: "action",
            label: t("text-recognition-qr-message"),
            url: url.toString(),
          }
        : unsupported(t("text-recognition-qr-message-incomplete"));
    default:
      return unsupported(
        t("text-recognition-qr-unsupported-action", {
          scheme: url.protocol.slice(0, -1),
        }),
      );
  }
};

// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
  label: "Unsupported QR";
  reason: string;
};

export type QrPayload = ActionPayload | InformationPayload | UnsupportedPayload;

const unsupported = (reason: string): UnsupportedPayload => ({
  kind: "unsupported",
  label: "Unsupported QR",
  reason,
});

const structuredPayload = (content: string): QrPayload | undefined => {
  if (/^WIFI:/iu.test(content)) {
    const ssid = /(?:^|;)S:((?:\\.|[^;])*)/iu.exec(content.slice(5))?.[1];
    return ssid
      ? { kind: "information", label: "Wi-Fi network" }
      : unsupported("Wi-Fi QR is missing a network name.");
  }
  if (/^BEGIN:VCARD\b/iu.test(content))
    return /END:VCARD\s*$/iu.test(content)
      ? { kind: "information", label: "Contact card" }
      : unsupported("Contact QR is incomplete.");
  if (/^BEGIN:(?:VCALENDAR|VEVENT)\b/iu.test(content))
    return /END:(?:VCALENDAR|VEVENT)\s*$/iu.test(content)
      ? { kind: "information", label: "Calendar event" }
      : unsupported("Calendar QR is incomplete.");
};

export const classifyQrPayload = (
  rawContent: string,
  decodeError?: string,
): QrPayload => {
  if (decodeError) return unsupported(decodeError);
  const content = rawContent.trim();
  if (!content) return unsupported("QR code has no content.");
  const structured = structuredPayload(content);
  if (structured) return structured;
  if (!/^[a-z][a-z\d+.-]*:/iu.test(content))
    return { kind: "information", label: "Text" };

  let url: URL;
  try {
    url = new URL(content);
  } catch {
    return unsupported("QR code contains a malformed action.");
  }

  switch (url.protocol) {
    case "http:":
    case "https:":
      return url.hostname
        ? {
            action: "link",
            kind: "action",
            label: "Open link",
            url: url.toString(),
          }
        : unsupported("Web QR is missing a destination.");
    case "tel:":
      return url.pathname
        ? {
            action: "phone",
            kind: "action",
            label: "Call number",
            url: url.toString(),
          }
        : unsupported("Phone QR is missing a number.");
    case "mailto:":
      return url.pathname
        ? {
            action: "email",
            kind: "action",
            label: "Compose email",
            url: url.toString(),
          }
        : unsupported("Email QR is missing a recipient.");
    case "sms:":
      return url.pathname
        ? {
            action: "sms",
            kind: "action",
            label: "Compose message",
            url: url.toString(),
          }
        : unsupported("Message QR is missing a recipient.");
    default:
      return unsupported(
        `The ${url.protocol.slice(0, -1)} action is not supported.`,
      );
  }
};

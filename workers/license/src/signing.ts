// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

const ALGORITHM = { name: "ECDSA", namedCurve: "P-256" } as const;

/** The claims a record vouches for, as the app reads them
 * (`src-tauri/crates/licensing/src/record.rs`). */
export type Claims = {
  entitlements: string[];
  keyHash: string;
  installationId: string;
  issuedAt: number;
  email?: string;
};

export type SignedRecord = { payload: string; signature: string };

/**
 * Signs the claims with the private key the app's public key pairs with
 * (`LICENSE_SIGNING_KEY`, PKCS#8 in base64). The payload is sent as the exact
 * bytes signed, and the signature in WebCrypto's own form, r and s of 32
 * bytes each, which the app reads as it is.
 */
export async function signRecord(
  claims: Claims,
  signingKey: string,
): Promise<SignedRecord> {
  const key = await crypto.subtle.importKey(
    "pkcs8",
    fromBase64(signingKey),
    ALGORITHM,
    false,
    ["sign"],
  );
  const payload = new TextEncoder().encode(JSON.stringify(claims));
  const signature = await crypto.subtle.sign(
    { name: "ECDSA", hash: "SHA-256" },
    key,
    payload,
  );
  return {
    payload: toBase64(payload),
    signature: toBase64(new Uint8Array(signature)),
  };
}

/** SHA-256 of the key as lowercase hex, as the app computes it. */
export async function keyHash(licenseKey: string): Promise<string> {
  const digest = await crypto.subtle.digest(
    "SHA-256",
    new TextEncoder().encode(licenseKey),
  );
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function toBase64(bytes: Uint8Array): string {
  return btoa(String.fromCharCode(...bytes));
}

function fromBase64(text: string): Uint8Array<ArrayBuffer> {
  return Uint8Array.from(atob(text.trim()), (char) => char.charCodeAt(0));
}

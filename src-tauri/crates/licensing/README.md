<!--
SPDX-FileCopyrightText: 2026 overpolish
SPDX-License-Identifier: Elastic-2.0
-->

# Licensing

Who may use the paid tools: Ruler, References and Glide. The crate decides; the app sends the requests, saves the state (`LicenseState`) and passes the time in.

## Whether a tool may open

All three tools share one 30-day trial, which starts the first time any of them is used. After it, each tool needs a key bought for it. A key held here counts only while its record verifies: signed by the licence server, for this installation and for that key.

```mermaid
flowchart TD
  start["Ruler, References or Glide starts"] --> key{"A held key's record verifies<br/>and grants this tool?"}
  key -- "Yes" --> licensed["Opens: licensed"]
  key -- "No" --> started{"Trial started?"}
  started -- "No" --> begin["Trial starts now, for all three tools"] --> trial["Opens: in trial"]
  started -- "Yes" --> within{"Less than 30 days since it started?"}
  within -- "Yes" --> trial
  within -- "No" --> expired["Expired: needs a key for this tool"]
```

## Adding a key, and checking it again

The app asks the licence server when a key is added, when a held key has not been confirmed for 21 days, and when a held record stops verifying (after a new server key, for example). Only the server's two named refusals take a key away; without a clear answer the key stays, so a tool never closes for someone offline. Payhip disables a key when its order is refunded, which reaches the app at its next check.

```mermaid
sequenceDiagram
  participant App
  participant Server as Licence server
  participant Payhip
  App->>Server: POST /activate {licenseKey, installationId}
  Server->>Payhip: Verify the key against each tool's product
  Payhip-->>Server: Enabled, disabled or unknown
  alt Key is good
    Server-->>App: 200 and a signed record
    App->>App: Check the signature offline, keep the record
  else Unknown or disabled
    Server-->>App: 400 invalid_key or 403 disabled
    App->>App: Drop the key
  else Anything else: offline, a timeout, a server error
    App->>App: Keep the key, ask again next launch
  end
```

The server's answer is fixed by `ActivationOutcome::from_response`. A record is the claims' JSON (`entitlements`, `keyHash`, `installationId`, `issuedAt`, optional `email`) and a P-256 signature over its exact bytes, both base64; the signature is r and s of 32 bytes each, as WebCrypto signs on the server. The test in `src/tests/contract.rs` holds a record WebCrypto signed, so a change on either side that breaks the other fails there.

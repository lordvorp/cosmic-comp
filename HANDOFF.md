# Handoff: COSMIC InputCapture + Clipboard for Deskflow

**Audience:** [DennisFury](https://github.com/DennisFury) (author of the original InputCapture PRs) and anyone championing Deskflow/Synergy on COSMIC Wayland.

**Framing — second half / finishing the job.** Your InputCapture PRs ([cosmic-comp#2853](https://github.com/pop-os/cosmic-comp/pull/2853), [portal-cosmic#369](https://github.com/pop-os/xdg-desktop-portal-cosmic/pull/369)) scaffolded the real path: consent, EIS, barriers, seat ownership. Deskflow and the portal *specs* were already willing to talk. What remained were **empty sockets** — `clipboard_enabled: false`, no Clipboard impl, no compositor bridge to selection — so the conversation never happened. This handoff **fills those gaps**. Cursor-hiding was UI sugar on top; the **core** is now strung together because the primitives (ext-data-control, smithay selection, portal Clipboard sequence, Deskflow’s libportal calls) were already there waiting.

**Intent:** Barriers and keyboard/pointer worked; CLIPBOARD did not. These patches complete that stack. **Please take them upstream to Pop/COSMIC** — Rob is not lobbying maintainers.

Deskflow itself needs **no code change** for CLIPBOARD. It already calls the portal correctly.

---

## Symptom (before this work)

On barrier activation, Deskflow 1.27 (Wayland) calls `xdp_session_get_selection_mime_types`. That list was empty, so the server logged `no current clipboard selection` and marshalled an empty clipboard (`size=4` = zero format count). Clients stored empty packets. Pointer/keyboard capture was fine.

## Root cause

| Layer | Gap |
|-------|-----|
| `xdg-desktop-portal` (frontend) | Noble stock **1.18.4** has no Clipboard-on-InputCapture. Need **≥ 1.21** (e.g. tag `1.21.1`). |
| `xdg-desktop-portal-cosmic` | InputCapture `Start` returned `clipboard_enabled: false`; no `org.freedesktop.impl.portal.Clipboard` / ext-data-control backend. |
| `cosmic-comp` | Private InputCapture D-Bus had no path tying the portal session to compositor selection (your PR covered EIS/barriers; clipboard is separate). |

Deskflow / libportal were waiting on a conversation the DE never finished.

## What we shipped (lordvorp forks)

| Repo | Branch | Tip | Baseline |
|------|--------|-----|----------|
| [lordvorp/cosmic-comp](https://github.com/lordvorp/cosmic-comp) | `inputcapture-on-41497b42` | `9b293983` | `41497b42` (pop-os master at cut) |
| [lordvorp/xdg-desktop-portal-cosmic](https://github.com/lordvorp/xdg-desktop-portal-cosmic) | `inputcapture-on-4902f57` | `0d04533` | `9d35e63` (rebased onto current master) |

Compare URLs:

- Comp: https://github.com/lordvorp/cosmic-comp/compare/41497b42...inputcapture-on-41497b42
- Portal: https://github.com/lordvorp/xdg-desktop-portal-cosmic/compare/9d35e63...inputcapture-on-4902f57

Patches under `handoff/patches/` in each repo (and mirrored as uniquely named files in release assets):

```text
41497b42..9b293983  →  cosmic-comp InputCapture D-Bus + unit tests + handoff docs
9d35e63..0d04533    →  portal InputCapture + Clipboard (ext-data-control) + tests + patches
```

Your prior work lives on local tracking branches `pr-2853` / `pr-369` (`d618128f` / `3fddb5a`). Cursor-hide from lordvorp (`1cc09b06`) was already folded into your compositor PR as polish. This handoff is the **clipboard second half** — wiring what the scaffold left empty — not a rewrite of your EIS/consent design.

## Stack (working CLIPBOARD path)

```text
Deskflow (stock) → xdg-desktop-portal ≥1.21 → xdg-desktop-portal-cosmic (Clipboard)
                                              → cosmic-comp (InputCapture D-Bus + selection)
```

- Portal `RequestClipboard` sets `clipboard_requested`; `Start` reports `clipboard_enabled` only when the ext-data-control backend is live.
- Selection read/write/announce follows the KDE-shaped portal Clipboard sequence.
- X11 PRIMARY / middle-click is **out of scope** here (portal API still single-channel for CLIPBOARD).

## Verify

Automated:

```bash
# cosmic-comp
cargo test --lib input_capture::tests

# xdg-desktop-portal-cosmic
cargo test input_capture::tests clipboard::tests
```

Manual (Pop noble / COSMIC):

1. Frontend ≥ 1.21.1, patched portal-cosmic, patched cosmic-comp (logout/login for compositor).
2. Restart order after portal rebuild: `org.freedesktop.impl.portal.desktop.cosmic.service`, then `xdg-desktop-portal.service`, then Deskflow. Do **not** kill the running compositor mid-session.
3. Consent InputCapture; copy text; cross barrier; paste on the peer. Expect clipboard size **> 4** and real mime types in Deskflow debug logs.
4. Reverse direction (copy on peer → paste on COSMIC).

Proven on sheila (Deskflow 1.27 server, COSMIC) ↔ cypher (client): text transfer after Clipboard impl; packet size e.g. 39 for a short string.

## What to ask Pop / distro for

1. Land InputCapture + Clipboard in `pop-os/cosmic-comp` and `pop-os/xdg-desktop-portal-cosmic` (your PRs were closed over AI/process concerns — evaluate the code; unit tests and a working Deskflow path exist).
2. Ship `xdg-desktop-portal` **≥ 1.21** so Clipboard works on InputCapture sessions (frontend PR territory; Ubuntu resolute already packages newer trees).

## Unofficial test .debs

GitHub Release on the cosmic-comp fork (assets include all three packages + patches):

https://github.com/lordvorp/cosmic-comp/releases/tag/inputcapture-clipboard-lordvorp1

Versions: `0.1+lordvorp1` / `1.10.0+lordvorp1` / `1.21.1-1lordvorp1`. **Not a Pop apt repo** — smoke-testing only. See `INSTALL.md` in the release.

## Contact

Patches: [lordvorp](https://github.com/lordvorp). Original InputCapture author: [DennisFury](https://github.com/DennisFury).

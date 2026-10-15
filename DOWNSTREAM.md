# DOWNSTREAM: public COSMIC bridge for Deskflow InputCapture + Clipboard

This file is the index. It is public. Point a reader, human or assistant, at this document and then at the compare URLs in each section.

**Who this is for:** [Chris Rizzitello](https://github.com/sithlord48) (Deskflow / the active Synergy line), who filed the COSMIC portal gap ([pop-os/xdg-desktop-portal-cosmic#197](https://github.com/pop-os/xdg-desktop-portal-cosmic/issues/197)) and who is filling the Wayland InputCapture + Clipboard path Deskflow already calls.

**Who this is not aimed at:** Pop / cosmic-comp maintainers. Rob is not opening Pop pull requests. An earlier InputCapture scaffold ([cosmic-comp#2853](https://github.com/pop-os/cosmic-comp/pull/2853), [xdg-desktop-portal-cosmic#369](https://github.com/pop-os/xdg-desktop-portal-cosmic/pull/369), author [DennisFury](https://github.com/DennisFury)) was closed. This bundle finishes the empty sockets those PRs left, on public forks, for Deskflow to use or champion.

**Read access:** no login and no collaborator invite. These forks are public:

- https://github.com/lordvorp/cosmic-comp
- https://github.com/lordvorp/xdg-desktop-portal-cosmic

Clone over HTTPS. That is read-only.

Deskflow 1.27 needs **no code change** for CLIPBOARD. On a barrier cross it already calls `xdp_session_get_selection_mime_types` / `xdp_session_set_selection`. COSMIC was the side that answered with an empty selection.

---

## Why this shape (KDE first, then Rust)

KDE is the reference desktop that already speaks this portal conversation.

`xdg-desktop-portal-kde` implements `org.freedesktop.impl.portal.Clipboard` as an `ext-data-control-v1` **client**: `RequestClipboard` grants the session, `SelectionRead` pipes `offer.receive`, `SetSelection` creates a data source, and the source's `send` events become `SelectionTransfer`. GNOME does the other shape (a Mutter clipboard proxy). COSMIC already has the Wayland side (`ext_data_control` / smithay selection in cosmic-comp). What it lacked was the portal object and the compositor D-Bus that let the portal frontend reach that selection.

The cosmic portal clipboard is that KDE role, written in Rust against the same protocol. It is not a new clipboard design. See `xdg-desktop-portal-cosmic/src/clipboard.rs` (the module comment states this) and KDE's `src/clipboard.cpp`.

InputCapture in cosmic-comp is the other half of the same idea: the compositor already had libei as an EI **sender** (inject events). InputCapture needs an EIS **receiver** (barriers, zones, seat, then hand physical input to the portal client). `reis` supplies the protocol. The new code is the policy and the D-Bus surface Deskflow's portal session expects.

X11 PRIMARY / middle-click paste is out of scope. The portal clipboard API here is one channel: CLIPBOARD.

---

## Section 1 — cosmic-comp: InputCapture D-Bus

**Repo:** https://github.com/lordvorp/cosmic-comp  
**Branch:** `inputcapture-on-41497b42`  
**Deskflow compare** (starts *after* the unrelated resize-cursor commits):  
https://github.com/lordvorp/cosmic-comp/compare/10ebfd13...inputcapture-on-41497b42

| Commit | What |
|--------|------|
| `13fd7cd7` | InputCapture D-Bus: EIS receiver, zones, barriers, enable/disable/release, session lock suspend |
| `0b0107ca` | Regression tests: capability reject, enable/disable, release activation id, barrier NaN |
| `67778a4d` `1285f906` `db60792d` | This handoff (`HANDOFF.md`, `handoff/patches/`) |

Also in this range, and part of **this** section: hide the local cursor while InputCapture is active (`hidden_for_input_capture` in `src/backend/render/cursor.rs`). Captured motion must not reveal the sprite or re-arm idle-hide. That is pointer-left-the-machine behavior, the same idea as the earlier `hide-cursor-during-input-capture` branch (`1cc09b06`).

Main files: `src/input_capture.rs`, `src/dbus/input_capture.rs`, `src/input/mod.rs`.

Patches: `handoff/patches/`.

```bash
cargo test --lib input_capture::tests
```

Longer narrative: [HANDOFF.md](HANDOFF.md).

---

## Section 2 — xdg-desktop-portal-cosmic: InputCapture portal + Clipboard

**Repo:** https://github.com/lordvorp/xdg-desktop-portal-cosmic  
**Branch:** `inputcapture-on-4902f57`  
**Compare:** https://github.com/lordvorp/xdg-desktop-portal-cosmic/compare/9d35e63...inputcapture-on-4902f57

This branch has **no** resize-cursor commits.

| Commit | What |
|--------|------|
| `95ab919` | InputCapture portal (consent, EIS fd to cosmic-comp) and Clipboard via ext-data-control |
| `1bd6614` | Tests: clipboard gates, mime sanitize (max 64 types / 255 chars), session grant/drop |
| `0d04533` | `handoff/patches/` export |

`Start` reports `clipboard_enabled: true` only when the session called `RequestClipboard` **and** the ext-data-control backend is actually up (`start_clipboard_enabled` in `src/clipboard.rs`). An advertised clipboard that cannot read and write is worse than `clipboard_enabled: false`.

Selection read/write follows the KDE-shaped sequence above. One clipboard object serves InputCapture and RemoteDesktop.

```bash
cargo test input_capture::tests clipboard::tests
```

---

## Section 3 — xdg-desktop-portal frontend ≥ 1.21

Pop noble's packaged frontend **1.18.4** does not offer Clipboard on InputCapture sessions. Its `clipboard.c` only accepts RemoteDesktop. Clipboard-on-InputCapture landed upstream in 2026 and is in the **1.21.1** tag.

No Deskflow or cosmic-comp change fixes that. The host needs a frontend ≥ 1.21, then a restart of `xdg-desktop-portal.service` (and the cosmic impl portal) **without** killing the running compositor mid-session.

Unofficial packaging commit on the local tree `~/work/xdg-desktop-portal`, branch `packaging-1.21.1-lordvorp`, commit `5b884534`: debian metadata only, version `1.21.1-1lordvorp1`. It is a test package, not an Ubuntu or Pop archive. Smoke debs from the earlier handoff release:

https://github.com/lordvorp/cosmic-comp/releases/tag/inputcapture-clipboard-lordvorp1

---

## What was proven

Sheila: COSMIC on Pop 24.04, Deskflow **1.27** server, portal clipboard.  
Cypher: KDE Plasma, Deskflow client. TLS.

Before: barrier activation logged `no current clipboard selection` and sent `size=4` (one uint32 format count of zero).  
After: text crossed both ways. A short string showed up with clipboard size greater than 4 and real mime types.

---

## Public, and not a request to Deskflow

The compositor branch `inputcapture-on-41497b42` is stacked on two commits that fix **window-border resize cursor** behavior (the icon disappearing on press, and not coming back on release while the pointer is still on the border):

| Commit | Branch | What |
|--------|--------|------|
| `db2db0b8` | `resize-cursor` | Keep the resize cursor for the whole border drag |
| `10ebfd13` | `resize-cursor` | Restore the resize cursor when the drag ends and the pointer is still in the border region |

https://github.com/lordvorp/cosmic-comp/compare/742b800b...resize-cursor

Those commits are public on purpose. They are a separate conversation with cosmic-comp. They are not part of the Deskflow clipboard bridge. The Section 1 compare starts at `10ebfd13` so a review of the Deskflow work does not have to include them. Anyone may still read them.

---

## Tests and restart

1. Frontend ≥ 1.21.1, patched `xdg-desktop-portal-cosmic`, patched `cosmic-comp`. Log out and back in before expecting a new compositor binary.
2. Restart `org.freedesktop.impl.portal.desktop.cosmic.service`, then `xdg-desktop-portal.service`, then Deskflow.
3. Consent to InputCapture. Copy text. Cross the barrier. Paste on the peer. Expect size > 4.
4. Copy on the peer and cross back. COSMIC should claim the selection.

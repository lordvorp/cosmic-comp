// SPDX-License-Identifier: GPL-3.0-only

use cosmic_comp_config::input::{DeviceState, TouchpadOverride};

/// How a libinput device relates to a built-in touchpad.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadRelation {
    /// The touchpad node (`config_tap_finger_count() > 0`).
    Touchpad,
    /// Another node in that touchpad's libinput device group.
    Companion,
    /// Not part of a touchpad.
    Unrelated,
}

/// Send-events state forced by [`TouchpadOverride`], if any.
///
/// `None` leaves the normal per-device / default config in place.
/// Companions that cannot do libinput's external-mouse mode are fully
/// disabled while an external pointer is present, and enabled again when
/// it is gone.
pub fn override_state(
    relation: PadRelation,
    touchpad_override: TouchpadOverride,
    external_pointer_present: bool,
    supports_disable_on_external_mouse: bool,
) -> Option<DeviceState> {
    let covered = matches!(relation, PadRelation::Touchpad | PadRelation::Companion);
    if !covered {
        return None;
    }

    match touchpad_override {
        TouchpadOverride::None => None,
        TouchpadOverride::ForceDisable => Some(DeviceState::Disabled),
        TouchpadOverride::DisableOnExternalMouse => {
            if supports_disable_on_external_mouse {
                Some(DeviceState::DisabledOnExternalMouse)
            } else if external_pointer_present {
                Some(DeviceState::Disabled)
            } else {
                Some(DeviceState::Enabled)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_override_leaves_every_device_alone() {
        for relation in [
            PadRelation::Touchpad,
            PadRelation::Companion,
            PadRelation::Unrelated,
        ] {
            assert_eq!(
                override_state(relation, TouchpadOverride::None, true, true),
                None
            );
        }
    }

    #[test]
    fn force_disable_covers_the_touchpad_and_its_companion() {
        assert_eq!(
            override_state(
                PadRelation::Touchpad,
                TouchpadOverride::ForceDisable,
                false,
                true
            ),
            Some(DeviceState::Disabled)
        );
        assert_eq!(
            override_state(
                PadRelation::Companion,
                TouchpadOverride::ForceDisable,
                true,
                false
            ),
            Some(DeviceState::Disabled)
        );
        assert_eq!(
            override_state(
                PadRelation::Unrelated,
                TouchpadOverride::ForceDisable,
                true,
                false
            ),
            None
        );
    }

    #[test]
    fn external_mouse_uses_libinput_mode_when_the_device_supports_it() {
        assert_eq!(
            override_state(
                PadRelation::Touchpad,
                TouchpadOverride::DisableOnExternalMouse,
                false,
                true
            ),
            Some(DeviceState::DisabledOnExternalMouse)
        );
        assert_eq!(
            override_state(
                PadRelation::Companion,
                TouchpadOverride::DisableOnExternalMouse,
                true,
                true
            ),
            Some(DeviceState::DisabledOnExternalMouse)
        );
    }

    #[test]
    fn companion_without_libinput_mode_tracks_the_external_pointer() {
        assert_eq!(
            override_state(
                PadRelation::Companion,
                TouchpadOverride::DisableOnExternalMouse,
                true,
                false
            ),
            Some(DeviceState::Disabled)
        );
        assert_eq!(
            override_state(
                PadRelation::Companion,
                TouchpadOverride::DisableOnExternalMouse,
                false,
                false
            ),
            Some(DeviceState::Enabled)
        );
    }

    #[test]
    fn external_mouse_override_ignores_unrelated_pointers() {
        assert_eq!(
            override_state(
                PadRelation::Unrelated,
                TouchpadOverride::DisableOnExternalMouse,
                true,
                false
            ),
            None
        );
    }
}

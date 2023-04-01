// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::{model, openvr};
use std::*;

struct ControllerState {
    n_buttons: u32,
    mode: Option<mimizu::GraffitiMode>,
    projector: mimizu::StrokeProjector,
}

pub struct VrInput {
    states: [ControllerState; 2],
}

impl VrInput {
    pub fn new() -> VrInput {
        VrInput {
            states: [
                ControllerState {
                    n_buttons: 0,
                    mode: None,
                    projector: mimizu::StrokeProjector::new(),
                },
                ControllerState {
                    n_buttons: 0,
                    mode: None,
                    projector: mimizu::StrokeProjector::new(),
                },
            ],
        }
    }

    pub fn update(&mut self, openvr: &openvr::OpenVr, model: &mut model::Model) {
        let indices = [
            openvr.get_tracked_device_index_for_controller_role(
                openvr::TrackedControllerRole::LeftHand,
            ),
            openvr.get_tracked_device_index_for_controller_role(
                openvr::TrackedControllerRole::RightHand,
            ),
        ];
        let size = cmp::max(cmp::max(indices[0], indices[1]), 1) as usize + 1;
        let mut poses = vec![openvr::TrackedDevicePose::default(); size];
        let controllers = [
            openvr.get_controller_state_with_pose(
                openvr::TrackingUniverseOrigin::Standing,
                indices[0],
            ),
            openvr.get_controller_state_with_pose(
                openvr::TrackingUniverseOrigin::Standing,
                indices[1],
            ),
        ];
        openvr.get_device_to_absolute_tracking_pose(
            openvr::TrackingUniverseOrigin::Standing,
            0.0,
            &mut poses,
        );

        let mut n_buttons = [0; 2];
        for i in 0..2 {
            n_buttons[i] = (controllers[i].0.button_pressed
                & (openvr::BUTTON_MASK_GRIP | openvr::BUTTON_MASK_TRIGGER))
                .count_ones();
        }

        if n_buttons.iter().all(|n| *n == 2) && self.states.iter().any(|s| s.n_buttons < 2) {
            model.is_active ^= true;
            for state in self.states.iter_mut() {
                state.projector.clear();
                state.mode = None;
            }
        } else if model.is_active {
            let head = poses[0].device_to_absolute_tracking.to_nalgebra();
            for (i, state) in self.states.iter_mut().enumerate() {
                let pose_key = controllers[i].1.device_to_absolute_tracking.to_nalgebra();
                let pose_now = poses[indices[i] as usize]
                    .device_to_absolute_tracking
                    .to_nalgebra();
                match (n_buttons[i].cmp(&state.n_buttons), state.mode) {
                    (cmp::Ordering::Less, Some(mode)) => {
                        state.projector.feed(&pose_key, &head);
                        model.feed_stroke(&state.projector.stroke(), mode);
                        state.projector.clear();
                        state.mode = None;
                    }
                    (cmp::Ordering::Greater, _) => {
                        state.projector.clear();
                        state.mode = Some(match n_buttons[i] {
                            1 => mimizu::GraffitiMode::Alphabet,
                            2 => mimizu::GraffitiMode::Number,
                            _ => unreachable!(),
                        });
                        state.projector.feed(&pose_key, &head);
                        state.projector.feed(&pose_now, &head);
                    }
                    (cmp::Ordering::Equal, Some(_)) => {
                        state.projector.feed(&pose_now, &head);
                    }
                    _ => (),
                }
            }
        }

        for (i, state) in self.states.iter_mut().enumerate() {
            state.n_buttons = n_buttons[i];
        }
        model.current_strokes = [
            self.states[0].projector.stroke(),
            self.states[1].projector.stroke(),
        ];
    }
}

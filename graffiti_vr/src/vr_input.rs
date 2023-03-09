use crate::*;
use std::*;

type Vector2 = nalgebra::Vector2<f32>;

pub struct VrInput {
    system: openvr::System,
    buttons: [bool; 2],
    projector: [graffiti_3d::StrokeProjector; 2],
    strokes: collections::VecDeque<Vec<Vector2>>,
}

impl VrInput {
    pub fn new() -> io::Result<VrInput> {
        let system = openvr::System::new()?;
        Ok(VrInput {
            system: system,
            buttons: [false, false],
            projector: [
                graffiti_3d::StrokeProjector::new(),
                graffiti_3d::StrokeProjector::new(),
            ],
            strokes: collections::VecDeque::new(),
        })
    }

    pub fn update(&mut self) {
        //GetTrackedDeviceIndexForControllerRole
        let mut poses: [_; 3] = Default::default();
        self.system.get_device_to_absolute_tracking_pose(&mut poses);
        let controllers = [
            self.system.get_controller_state_with_pose(1),
            self.system.get_controller_state_with_pose(2),
        ];

        let head = poses[0].device_to_absolute_tracking.to_nalgebra();
        for i in 0..2 {
            let button = controllers[i].0.button_pressed
                & (openvr::BUTTON_MASK_GRIP | openvr::BUTTON_MASK_TRIGGER)
                != 0;
            if button != self.buttons[i] {
                let hand = controllers[i].1.device_to_absolute_tracking.to_nalgebra();
                self.projector[i].feed(&hand, &head);
            }
            if button {
                let hand = poses[i + 1].device_to_absolute_tracking.to_nalgebra();
                self.projector[i].feed(&hand, &head);
            }
            if (self.buttons[i], button) == (true, false) {
                self.strokes.push_back(self.projector[i].stroke());
                self.projector[i].clear();
            }
            self.buttons[i] = button;
        }
    }

    pub fn current_strokes(&self) -> [Vec<Vector2>; 2] {
        [self.projector[0].stroke(), self.projector[1].stroke()]
    }

    pub fn pop_stroke(&mut self) -> Option<Vec<Vector2>> {
        self.strokes.pop_front()
    }
}

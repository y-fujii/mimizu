use std::*;

pub const BUTTON_MASK_GRIP: u64 = 1 << 2;
pub const BUTTON_MASK_TRIGGER: u64 = 1 << 33;

// note: the structs in "openvr.h" are defined with "#pragma pack(8)".

#[derive(Clone, Default, Debug)]
#[repr(C)]
pub struct HmdVector3 {
    pub v: [f32; 3],
}

#[derive(Clone, Default, Debug)]
#[repr(C)]
pub struct HmdMatrix34 {
    pub m: [[f32; 4]; 3],
}

#[derive(Clone, Default, Debug)]
#[repr(C)]
pub struct VRControllerAxis {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Default, Debug)]
#[repr(C)]
pub struct VRControllerState {
    pub packet_num: u32,
    pub button_pressed: u64,
    pub button_touched: u64,
    pub axis: [VRControllerAxis; 5],
}

#[derive(Clone, Default, Debug)]
#[repr(C)]
pub struct TrackedDevicePose {
    pub device_to_absolute_tracking: HmdMatrix34,
    pub velocity: HmdVector3,
    pub angular_velocity: HmdVector3,
    pub tracking_result: u32,
    pub pose_is_valid: bool,
    pub device_is_connected: bool,
}

pub struct System {
    this: *mut ffi::c_void,
}

extern "C" {
    fn vr_init() -> *mut ffi::c_void;
    fn vr_get_device_to_absolute_tracking_pose(
        _: *mut ffi::c_void,
        _: *mut TrackedDevicePose,
        _: u32,
    );
    fn vr_get_controller_state_with_pose(
        _: *mut ffi::c_void,
        _: u32,
        _: *mut VRControllerState,
        _: &mut TrackedDevicePose,
    );
    fn vr_shutdown(_: *mut ffi::c_void);
}

impl HmdMatrix34 {
    pub fn to_nalgebra(&self) -> nalgebra::Matrix3x4<f32> {
        let m = &self.m;
        nalgebra::Matrix3x4::new(
            m[0][0], m[0][1], m[0][2], m[0][3], //
            m[1][0], m[1][1], m[1][2], m[1][3], //
            m[2][0], m[2][1], m[2][2], m[2][3], //
        )
    }
}

impl Drop for System {
    fn drop(&mut self) {
        unsafe { vr_shutdown(self.this) };
    }
}

impl System {
    pub fn new() -> Result<Self, io::Error> {
        let this = unsafe { vr_init() };
        if this.is_null() {
            Err(io::ErrorKind::Other.into())
        } else {
            Ok(System { this })
        }
    }

    pub fn get_device_to_absolute_tracking_pose(&self, dst: &mut [TrackedDevicePose]) {
        unsafe {
            vr_get_device_to_absolute_tracking_pose(self.this, dst.as_mut_ptr(), dst.len() as u32)
        };
    }

    pub fn get_controller_state_with_pose(&self, n: u32) -> (VRControllerState, TrackedDevicePose) {
        let mut state = Default::default();
        let mut pose = Default::default();
        unsafe { vr_get_controller_state_with_pose(self.this, n, &mut state, &mut pose) };
        (state, pose)
    }
}

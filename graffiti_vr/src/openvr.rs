use std::*;

pub const BUTTON_MASK_GRIP: u64 = 1 << 2;
pub const BUTTON_MASK_TRIGGER: u64 = 1 << 33;
pub const OVERLAY_FLAGS_PREMULTIPLIED: u32 = 1 << 21;

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

pub struct Overlay {
    this: *mut ffi::c_void,
}

extern "C" {
    fn vr_init() -> bool;
    fn vr_shutdown();

    fn vr_system() -> *mut ffi::c_void;
    fn vr_system_get_device_to_absolute_tracking_pose(
        _: *mut ffi::c_void,
        _: *mut TrackedDevicePose,
        _: u32,
    );
    fn vr_system_get_controller_state_with_pose(
        _: *mut ffi::c_void,
        _: u32,
        _: *mut VRControllerState,
        _: *mut TrackedDevicePose,
    );

    fn vr_overlay() -> *mut ffi::c_void;
    fn vr_overlay_create(_: *mut ffi::c_void, _: *const u8, _: *const u8) -> usize;
    fn vr_overlay_set_flag(_: *mut ffi::c_void, _: usize, _: u32, _: bool) -> bool;
    fn vr_overlay_set_width_in_meters(_: *mut ffi::c_void, _: usize, _: f32) -> bool;
    fn vr_overlay_set_transform_tracked_device_relative(
        _: *mut ffi::c_void,
        _: usize,
        _: u32,
        _: *const HmdMatrix34,
    ) -> bool;
    fn vr_overlay_set_texture(_: *mut ffi::c_void, _: usize, _: usize) -> bool;
    fn vr_overlay_show(_: *mut ffi::c_void, _: usize) -> bool;
    fn vr_overlay_destroy(_: *mut ffi::c_void, _: usize) -> bool;
}

pub fn init() -> bool {
    unsafe { vr_init() }
}

pub fn shutdown() {
    unsafe { vr_shutdown() }
}

impl HmdMatrix34 {
    pub fn from_nalgebra(m: &nalgebra::Matrix3x4<f32>) -> Self {
        let m = [
            [m[(0, 0)], m[(0, 1)], m[(0, 2)], m[(0, 3)]],
            [m[(1, 0)], m[(1, 1)], m[(1, 2)], m[(1, 3)]],
            [m[(2, 0)], m[(2, 1)], m[(2, 2)], m[(2, 3)]],
        ];
        HmdMatrix34 { m: m }
    }

    pub fn to_nalgebra(&self) -> nalgebra::Matrix3x4<f32> {
        let m = &self.m;
        nalgebra::Matrix3x4::new(
            m[0][0], m[0][1], m[0][2], m[0][3], //
            m[1][0], m[1][1], m[1][2], m[1][3], //
            m[2][0], m[2][1], m[2][2], m[2][3], //
        )
    }
}

impl System {
    pub fn new() -> Self {
        let this = unsafe { vr_system() };
        assert!(!this.is_null());
        System { this: this }
    }

    pub fn get_device_to_absolute_tracking_pose(&self, dst: &mut [TrackedDevicePose]) {
        unsafe {
            vr_system_get_device_to_absolute_tracking_pose(
                self.this,
                dst.as_mut_ptr(),
                dst.len() as u32,
            )
        };
    }

    pub fn get_controller_state_with_pose(&self, n: u32) -> (VRControllerState, TrackedDevicePose) {
        let mut state = Default::default();
        let mut pose = Default::default();
        unsafe { vr_system_get_controller_state_with_pose(self.this, n, &mut state, &mut pose) };
        (state, pose)
    }
}

impl Overlay {
    pub fn new() -> Self {
        let this = unsafe { vr_overlay() };
        assert!(!this.is_null());
        Overlay { this: this }
    }

    pub fn create(&self, key: &[u8], name: &[u8]) -> usize {
        assert!(key.last() == Some(&b'\0'));
        assert!(name.last() == Some(&b'\0'));
        unsafe { vr_overlay_create(self.this, key.as_ptr(), name.as_ptr()) }
    }

    pub fn set_flag(&self, handle: usize, flag: u32, enabled: bool) -> bool {
        unsafe { vr_overlay_set_flag(self.this, handle, flag, enabled) }
    }

    pub fn set_width_in_meters(&self, handle: usize, width: f32) -> bool {
        unsafe { vr_overlay_set_width_in_meters(self.this, handle, width) }
    }

    pub fn set_transform_tracked_device_relative(
        &self,
        handle: usize,
        device: u32,
        transform: &HmdMatrix34,
    ) -> bool {
        unsafe {
            vr_overlay_set_transform_tracked_device_relative(self.this, handle, device, transform)
        }
    }

    pub fn set_texture(&self, handle: usize, tex_handle: usize) -> bool {
        unsafe { vr_overlay_set_texture(self.this, handle, tex_handle) }
    }

    pub fn show(&self, handle: usize) -> bool {
        unsafe { vr_overlay_show(self.this, handle) }
    }

    pub fn destroy(&self, handle: usize) -> bool {
        unsafe { vr_overlay_destroy(self.this, handle) }
    }
}

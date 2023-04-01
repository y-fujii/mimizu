// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

pub const BUTTON_MASK_GRIP: u64 = 1 << 2;
pub const BUTTON_MASK_TRIGGER: u64 = 1 << 33;
pub const OVERLAY_FLAGS_IS_PREMULTIPLIED: u32 = 1 << 21;

#[repr(C)]
pub enum ApplicationType {
    Overlay = 2,
}

#[repr(C)]
pub enum TrackingUniverseOrigin {
    Standing = 1,
}

#[repr(C)]
pub enum TrackedControllerRole {
    LeftHand = 1,
    RightHand = 2,
}

#[repr(C)]
pub enum TextureType {
    OpenGL = 1,
}

#[repr(C)]
pub enum ColorSpace {
    Auto = 0,
}

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

#[repr(C)]
pub struct Texture {
    pub handle: usize,
    pub type_: TextureType,
    pub color_space: ColorSpace,
}

#[repr(C)]
struct SystemFnTable {
    _dummy_0: [usize; 11],
    get_device_to_absolute_tracking_pose: extern "system" fn(i32, f32, *mut TrackedDevicePose, u32),
    _dummy_1: [usize; 5],
    get_tracked_device_index_for_controller_role: extern "system" fn(i32) -> i32,
    _dummy_2: [usize; 16],
    get_controller_state_with_pose:
        extern "system" fn(i32, i32, *mut VRControllerState, u32, *mut TrackedDevicePose) -> bool,
}

#[repr(C)]
struct OverlayFnTable {
    _dummy_0: [usize; 1],
    create_overlay: extern "system" fn(*const u8, *const u8, *mut u64) -> i32,
    destroy_overlay: extern "system" fn(u64) -> i32,
    _dummy_1: [usize; 7],
    set_overlay_flag: extern "system" fn(u64, u32, bool) -> i32,
    _dummy_2: [usize; 10],
    set_overlay_width_in_meters: extern "system" fn(u64, f32) -> i32,
    _dummy_3: [usize; 12],
    set_overlay_transform_tracked_device_relative:
        extern "system" fn(u64, u32, *const HmdMatrix34) -> i32,
    _dummy_4: [usize; 8],
    show_overlay: extern "system" fn(u64) -> i32,
    hide_overlay: extern "system" fn(u64) -> i32,
    _dummy_5: [usize; 15],
    set_overlay_texture: extern "system" fn(u64, *const Texture) -> i32,
}

pub struct OpenVr {
    system: *const SystemFnTable,
    overlay: *const OverlayFnTable,
}

#[link(name = "openvr_api")]
extern "C" {
    fn VR_InitInternal2(_: *mut i32, _: i32, _: *const u8) -> u32;
    fn VR_ShutdownInternal();
    fn VR_GetGenericInterface(_: *const u8, _: *mut i32) -> *const ffi::c_void;
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

impl Drop for OpenVr {
    fn drop(&mut self) {
        unsafe { VR_ShutdownInternal() }
    }
}

impl OpenVr {
    pub fn new(app_type: ApplicationType) -> Option<Self> {
        let mut err = 0;
        unsafe { VR_InitInternal2(&mut err, app_type as i32, ptr::null()) };
        if err != 0 {
            return None;
        }
        let system =
            unsafe { VR_GetGenericInterface(b"FnTable:IVRSystem_022\0".as_ptr(), &mut err) }
                as *const SystemFnTable;
        if system.is_null() {
            return None;
        }
        let overlay =
            unsafe { VR_GetGenericInterface(b"FnTable:IVROverlay_026\0".as_ptr(), &mut err) }
                as *const OverlayFnTable;
        if overlay.is_null() {
            return None;
        }
        Some(OpenVr {
            system: system,
            overlay: overlay,
        })
    }

    pub fn get_tracked_device_index_for_controller_role(&self, role: TrackedControllerRole) -> i32 {
        unsafe { ((*self.system).get_tracked_device_index_for_controller_role)(role as i32) }
    }

    pub fn get_device_to_absolute_tracking_pose(
        &self,
        origin: TrackingUniverseOrigin,
        secs: f32,
        dst: &mut [TrackedDevicePose],
    ) {
        unsafe {
            ((*self.system).get_device_to_absolute_tracking_pose)(
                origin as i32,
                secs,
                dst.as_mut_ptr(),
                dst.len() as u32,
            )
        };
    }

    pub fn get_controller_state_with_pose(
        &self,
        origin: TrackingUniverseOrigin,
        n: i32,
    ) -> (VRControllerState, TrackedDevicePose) {
        let mut state = Default::default();
        let mut pose = Default::default();
        unsafe {
            ((*self.system).get_controller_state_with_pose)(
                origin as i32,
                n,
                &mut state,
                mem::size_of::<VRControllerState>() as u32,
                &mut pose,
            )
        };
        (state, pose)
    }

    pub fn create_overlay(&self, key: &[u8], name: &[u8]) -> u64 {
        assert!(key.last() == Some(&b'\0'));
        assert!(name.last() == Some(&b'\0'));
        let mut handle = 0;
        unsafe { ((*self.overlay).create_overlay)(key.as_ptr(), name.as_ptr(), &mut handle) };
        handle
    }

    pub fn set_overlay_flag(&self, handle: u64, flag: u32, enabled: bool) -> bool {
        unsafe { ((*self.overlay).set_overlay_flag)(handle, flag, enabled) == 0 }
    }

    pub fn set_overlay_width_in_meters(&self, handle: u64, width: f32) -> bool {
        unsafe { ((*self.overlay).set_overlay_width_in_meters)(handle, width) == 0 }
    }

    pub fn set_overlay_transform_tracked_device_relative(
        &self,
        handle: u64,
        device: u32,
        transform: &HmdMatrix34,
    ) -> bool {
        unsafe {
            ((*self.overlay).set_overlay_transform_tracked_device_relative)(
                handle, device, transform,
            ) == 0
        }
    }

    pub fn set_overlay_texture(&self, handle: u64, texture: &Texture) -> bool {
        unsafe { ((*self.overlay).set_overlay_texture)(handle, texture) == 0 }
    }

    pub fn show_overlay(&self, handle: u64) -> bool {
        unsafe { ((*self.overlay).show_overlay)(handle) == 0 }
    }

    pub fn hide_overlay(&self, handle: u64) -> bool {
        unsafe { ((*self.overlay).hide_overlay)(handle) == 0 }
    }

    pub fn destroy_overlay(&self, handle: u64) -> bool {
        unsafe { ((*self.overlay).destroy_overlay)(handle) == 0 }
    }
}

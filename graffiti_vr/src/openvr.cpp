#if defined(__MINGW32__)
	#include <openvr/openvr_mingw.hpp>
#else
	#include <openvr/openvr.h>
#endif


extern "C" vr::IVRSystem* vr_init() {
	vr::HmdError err;
	return vr::VR_Init(&err, vr::VRApplication_Overlay);
}

extern "C" void vr_get_device_to_absolute_tracking_pose(vr::IVRSystem* self, vr::TrackedDevicePose_t* poses, uint32_t n) {
	self->GetDeviceToAbsoluteTrackingPose(vr::TrackingUniverseStanding, 0.0, poses, n);
}

extern "C" void vr_get_controller_state(vr::IVRSystem* self, uint32_t index, vr::VRControllerState_t* state) {
	self->GetControllerState(index, state, sizeof(vr::VRControllerState_t));
}

extern "C" void vr_shutdown(vr::IVRSystem*) {
	vr::VR_Shutdown();
}

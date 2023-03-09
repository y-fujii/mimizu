#include <windows.h>


// ref. <https://learn.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timebeginperiod>.
// ref. <https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessinformation>.
extern "C" void set_windows_timer_precision() {
	PROCESS_POWER_THROTTLING_STATE ppts = {};
	ppts.Version = PROCESS_POWER_THROTTLING_CURRENT_VERSION;
	ppts.ControlMask = PROCESS_POWER_THROTTLING_IGNORE_TIMER_RESOLUTION;
	ppts.StateMask = 0;
	SetProcessInformation(GetCurrentProcess(), ProcessPowerThrottling, &ppts, sizeof(ppts));

	timeBeginPeriod(1);
}

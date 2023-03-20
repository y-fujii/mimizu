#if defined(_WIN32)

#include <cstdint>
#include <windows.h>


extern "C" bool sleep_100ns(std::int64_t t) {
	thread_local HANDLE timer = CreateWaitableTimerEx(NULL, NULL, CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS);
	if (timer == NULL) {
		return false;
	}

	LARGE_INTEGER time;
	time.QuadPart = -t;
	if (!SetWaitableTimer(timer, &time, 0, NULL, NULL, 0)) {
		return false;
	}

	if (WaitForSingleObject(timer, INFINITE) != WAIT_OBJECT_0) {
		return false;
	}

	return true;
}


#endif

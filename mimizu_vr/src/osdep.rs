// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

#[cfg(not(windows))]
pub use thread::sleep;

#[cfg(not(windows))]
pub fn emulate_key(_: char) {}

#[cfg(windows)]
pub fn sleep(dur: time::Duration) {
    use windows_sys::Win32::{Foundation::*, System::Threading::*};

    thread_local!(static TIMER: HANDLE = {
        let timer = unsafe {
            CreateWaitableTimerExA(
                ptr::null(),
                ptr::null(),
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_ALL_ACCESS,
            )
        };
        if timer.is_null() {
            panic!();
        }
        timer
    });

    TIMER.with(|timer| {
        let d = (dur.as_nanos() as i128 / -100) as i64;
        if unsafe { SetWaitableTimer(*timer, &d, 0, None, ptr::null(), 0) } == 0 {
            panic!();
        }
        if unsafe { WaitForSingleObject(*timer, INFINITE) } != WAIT_OBJECT_0 {
            panic!();
        }
    })
}

#[cfg(windows)]
pub fn emulate_key(ch: char) {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

    // XXX
    let (vk, scan, flags) = match ch {
        '\x08' => (VK_BACK, 0, 0),
        '\n' => (VK_RETURN, 0, 0),
        '←' => (VK_LEFT, 0, 0),
        '→' => (VK_RIGHT, 0, 0),
        _ => {
            let mut buf = [0];
            ch.encode_utf16(&mut buf);
            (0, buf[0], KEYEVENTF_UNICODE)
        }
    };
    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan,
                    dwFlags: flags | KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];
    if unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            mem::size_of::<INPUT>() as i32,
        )
    } == 0
    {
        panic!();
    }
}

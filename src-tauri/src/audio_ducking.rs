use std::sync::Once;
use std::thread;
use std::time::Duration;

use windows::core::Interface;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;

static COM_INIT: Once = Once::new();

fn init_com() {
    COM_INIT.call_once(|| unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok();
    });
}

pub struct AudioDucker;

impl AudioDucker {
    /// Duck all system audio sessions and restore after delay
    pub fn duck_all_sessions(volume: f32, restore_ms: u64) {
        init_com();

        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();

            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .unwrap();

            let manager: IAudioSessionManager2 =
                device.Activate(CLSCTX_ALL, None).unwrap();

            let sessions = manager.GetSessionEnumerator().unwrap();
            let count = sessions.GetCount().unwrap();

            for i in 0..count {
                let session = sessions.GetSession(i).unwrap();
                let volume_ctrl: ISimpleAudioVolume = session.cast().unwrap();
                let _ = volume_ctrl.SetMasterVolume(volume, std::ptr::null());
            }

            // Restore volume after delay
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(restore_ms));

                let enumerator: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();

                let device = enumerator
                    .GetDefaultAudioEndpoint(eRender, eConsole)
                    .unwrap();

                let manager: IAudioSessionManager2 =
                    device.Activate(CLSCTX_ALL, None).unwrap();

                let sessions = manager.GetSessionEnumerator().unwrap();
                let count = sessions.GetCount().unwrap();

                for i in 0..count {
                    let session = sessions.GetSession(i).unwrap();
                    let volume_ctrl: ISimpleAudioVolume = session.cast().unwrap();
                    let _ = volume_ctrl.SetMasterVolume(1.0, std::ptr::null());
                }
            });
        }
    }
}

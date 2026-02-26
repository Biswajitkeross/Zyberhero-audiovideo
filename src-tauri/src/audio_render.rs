use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::PropertiesSystem::IPropertyStore;
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::core::{PCWSTR, PROPVARIANT};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use crate::delay_buffer::DelayBuffer;

// PKEY_Device_FriendlyName: {a45c254e-df1c-4efd-8020-67d146a850e0}, 14
const PKEY_DEVICE_FRIENDLY_NAME: windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY = 
    windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY {
        fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
        pid: 14,
    };

pub fn enumerate_output_devices() -> Vec<(String, String)> {
    let mut devices = Vec::new();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED).ok();
        
        // Block to ensure COM objects are dropped before CoUninitialize
        let _ = (|| -> windows::core::Result<()> {
            let enumerator: IMMDeviceEnumerator = 
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            
            let collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
            let count = collection.GetCount()?;

            for i in 0..count {
                let device = collection.Item(i)?;
                let id = device.GetId()?;
                let id_str = id.to_string().unwrap_or_default();

                let store: IPropertyStore = device.OpenPropertyStore(STGM_READ)?;
                let friendly_name_prop: PROPVARIANT = store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME)?;
                
                // Use safe helper to extract string from PROPVARIANT
                let name = match PropVariantToStringAlloc(&friendly_name_prop) {
                    Ok(friendly_name) => {
                        let s = friendly_name.to_string().unwrap_or_else(|_| "Unknown Device".to_string());
                        // Free the allocated string
                        let _ = windows::Win32::System::Com::CoTaskMemFree(Some(friendly_name.as_ptr() as *const _)); 
                        if s.is_empty() { "Unknown Device".to_string() } else { s }
                    },
                    Err(_) => "Unknown Device".to_string(),
                };

                devices.push((id_str, name));
            }
            Ok(())
        })();

        // CoUninitialize(); // We might leave this initialized for the thread
    }
    devices
}


pub fn get_default_device_sample_rate() -> Option<u32> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED).ok();
        let result = (|| -> windows::core::Result<u32> {
             let enumerator: IMMDeviceEnumerator = 
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
             let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
             let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
             let mix_format_ptr = audio_client.GetMixFormat()?;
             let mix_format = *mix_format_ptr;
             let sample_rate = mix_format.nSamplesPerSec;
             windows::Win32::System::Com::CoTaskMemFree(Some(mix_format_ptr as *const _));
             Ok(sample_rate)
        })();
        result.ok()
    }
}

pub fn start_audio_render(
    buffer: Arc<DelayBuffer>,
    stop: Arc<AtomicBool>,
    _target_sample_rate: u32,
    device_id: Option<String>, 
) -> windows::core::Result<()> {

    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED).ok();

        let device_enum: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        let device = if let Some(id) = device_id {
            // Use specific device
            let mut wide_id: Vec<u16> = id.encode_utf16().collect();
            wide_id.push(0);
            println!("🔈 [AudioRender] Attempting to open specific device ID: {}", id);
            device_enum.GetDevice(PCWSTR::from_raw(wide_id.as_ptr()))?
        } else {
            // Use default
            println!("🔈 [AudioRender] Using System Default Output Device");
            device_enum.GetDefaultAudioEndpoint(eRender, eMultimedia)?
        };

        // Get friendly name for debug
        let store: IPropertyStore = device.OpenPropertyStore(STGM_READ)?;
        let friendly_name_prop: PROPVARIANT = store.GetValue(&PKEY_DEVICE_FRIENDLY_NAME)?;
        let device_name = match PropVariantToStringAlloc(&friendly_name_prop) {
            Ok(name) => {
                let s = name.to_string().unwrap_or_else(|_| "Unknown".to_string());
                let _ = windows::Win32::System::Com::CoTaskMemFree(Some(name.as_ptr() as *const _));
                s
            },
            Err(_) => "Unknown Device".to_string(),
        };
        println!("🔈 [AudioRender] Activated Device: '{}'", device_name);

        let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
        
        let mix_format_ptr = audio_client.GetMixFormat()?;
        let mix_format = *mix_format_ptr;
        
        let sample_rate = mix_format.nSamplesPerSec;
        let channels_count = mix_format.nChannels;
        
        println!("🔈 [AudioRender] Mix Format: {} Hz, {} Channels", sample_rate, channels_count);

        audio_client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            0, 
            10_000_000, 
            0,
            mix_format_ptr,
            None,
        )?;

        let render_client: IAudioRenderClient = audio_client.GetService()?;
        audio_client.Start()?;
        
        let channels = channels_count as usize;
        println!("🔈 [AudioRender] Playback Started on '{}'", device_name);

        while !stop.load(Ordering::Relaxed) {
            let padding = audio_client.GetCurrentPadding()?;
            let buffer_size = audio_client.GetBufferSize()?;
            let frames_available = buffer_size - padding;

            if frames_available > 0 {
                // Get Mono samples from Ring Buffer
                let samples = buffer.pop(frames_available as usize);
                
                if !samples.is_empty() {
                    let ptr = render_client.GetBuffer(samples.len() as u32)?;
                    let out = std::slice::from_raw_parts_mut(
                        ptr as *mut f32,
                        samples.len() * channels, 
                    );

                    // Mix Mono -> Stereo/Surround
                    for (i, sample) in samples.iter().enumerate() {
                        for c in 0..channels {
                            out[i * channels + c] = *sample;
                        }
                    }

                    render_client.ReleaseBuffer(samples.len() as u32, 0)?;
                } else {
                    thread::sleep(Duration::from_millis(5));
                }
            } else {
                thread::sleep(Duration::from_millis(5));
            }
        }

        audio_client.Stop()?;
        CoUninitialize();
        windows::Win32::System::Com::CoTaskMemFree(Some(mix_format_ptr as *const _));
        Ok(())
    }
}
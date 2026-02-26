//! Blur Overlay for Windows - Creates a fullscreen warning overlay

use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use winapi::um::wingdi::{CreateSolidBrush, RGB, DeleteObject};
use winapi::um::winuser::{
    RegisterClassW, CreateWindowExW, ShowWindow, UpdateWindow, GetMessageW, 
    TranslateMessage, DispatchMessageW, DefWindowProcW, PostQuitMessage,
    DestroyWindow, GetSystemMetrics, SetLayeredWindowAttributes,
    WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TOOLWINDOW, WS_POPUP,
    SW_SHOW, SW_HIDE, SM_CXSCREEN, SM_CYSCREEN, LWA_ALPHA, WM_DESTROY, WM_PAINT,
    CS_HREDRAW, CS_VREDRAW, MSG, WNDCLASSW, BeginPaint, EndPaint, FillRect, PAINTSTRUCT,
};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::shared::windef::HWND;
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT};

pub struct BlurOverlay {
    hwnd: AtomicUsize, // Store HWND as usize for Send/Sync
    visible: Arc<AtomicBool>,
}

unsafe impl Send for BlurOverlay {}
unsafe impl Sync for BlurOverlay {}

impl BlurOverlay {
    pub fn new() -> Result<Self, String> {
        let visible = Arc::new(AtomicBool::new(false));
        let hwnd_storage = Arc::new(AtomicUsize::new(0));
        let hwnd_clone = hwnd_storage.clone();

        // Create window in a separate thread to handle message pump
        thread::spawn(move || {
            unsafe {
                let hinstance = GetModuleHandleW(null_mut());
                
                let class_name: Vec<u16> = "BlurOverlay\0".encode_utf16().collect();
                
                let wc = WNDCLASSW {
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(window_proc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: hinstance,
                    hIcon: null_mut(),
                    hCursor: null_mut(),
                    hbrBackground: CreateSolidBrush(RGB(0, 0, 0)), // Pure black
                    lpszMenuName: null_mut(),
                    lpszClassName: class_name.as_ptr(),
                };

                RegisterClassW(&wc);

                let width = GetSystemMetrics(SM_CXSCREEN);
                let height = GetSystemMetrics(SM_CYSCREEN);

                let hwnd = CreateWindowExW(
                    WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                    class_name.as_ptr(),
                    class_name.as_ptr(),
                    WS_POPUP,
                    0, 0, width, height,
                    null_mut(),
                    null_mut(),
                    hinstance,
                    null_mut(),
                );

                if !hwnd.is_null() {
                    // 100% opacity - completely blocks view
                    SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
                    hwnd_clone.store(hwnd as usize, Ordering::SeqCst);
                }

                // Message loop
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });

        // Wait for window creation
        thread::sleep(Duration::from_millis(200));
        
        let hwnd_val = hwnd_storage.load(Ordering::SeqCst);
        if hwnd_val == 0 {
            return Err("Failed to create overlay window".to_string());
        }

        Ok(Self { 
            hwnd: AtomicUsize::new(hwnd_val),
            visible 
        })
    }

    pub fn show(&self) {
        if !self.visible.swap(true, Ordering::SeqCst) {
            let hwnd = self.hwnd.load(Ordering::SeqCst) as HWND;
            unsafe {
                ShowWindow(hwnd, SW_SHOW);
                UpdateWindow(hwnd);
            }
        }
    }

    pub fn hide(&self) {
        if self.visible.swap(false, Ordering::SeqCst) {
            let hwnd = self.hwnd.load(Ordering::SeqCst) as HWND;
            unsafe {
                ShowWindow(hwnd, SW_HIDE);
            }
        }
    }
}

impl Drop for BlurOverlay {
    fn drop(&mut self) {
        let hwnd = self.hwnd.load(Ordering::SeqCst) as HWND;
        if !hwnd.is_null() {
            unsafe {
                DestroyWindow(hwnd);
            }
        }
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            
            let brush = CreateSolidBrush(RGB(0, 0, 0)); // Pure black
            FillRect(hdc, &ps.rcPaint, brush);
            DeleteObject(brush as *mut _);
            
            EndPaint(hwnd, &ps);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

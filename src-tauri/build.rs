fn main() {
  // Add Vosk library path for Windows
  #[cfg(target_os = "windows")]
  {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let vosk_lib_path = format!("{}\\vosk-win64-0.3.45", manifest_dir);
    println!("cargo:rustc-link-search=native={}", vosk_lib_path);
    
    // Also set the path for DLL loading at runtime
    println!("cargo:rerun-if-changed={}", vosk_lib_path);
  }
  
  tauri_build::build()
}

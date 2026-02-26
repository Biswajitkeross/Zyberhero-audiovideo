//! NSFW Detector using NudeNet ONNX model

use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

const INPUT_SIZE: usize = 320;
const EXPLICIT_THRESHOLD: f32 = 0.40;  // Lower threshold for explicit content (nudity)
const SUGGESTIVE_THRESHOLD: f32 = 0.50; // Threshold for suggestive content (bikini)

static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);

// NudeNet class indices
// EXPLICIT - Direct nudity (always flag)
const EXPLICIT_CLASSES: &[usize] = &[
    2,  // BUTTOCKS_EXPOSED
    3,  // FEMALE_BREAST_EXPOSED
    4,  // FEMALE_GENITALIA_EXPOSED
    6,  // ANUS_EXPOSED
    14, // MALE_GENITALIA_EXPOSED
];

// SUGGESTIVE - Bikini/swimwear content (flag for children's safety)
const SUGGESTIVE_CLASSES: &[usize] = &[
    0,  // FEMALE_GENITALIA_COVERED (bikini bottom)
    16, // FEMALE_BREAST_COVERED (bikini top)
    17, // BUTTOCKS_COVERED (bikini bottom from behind)
    13, // BELLY_EXPOSED (midriff showing)
];

#[allow(dead_code)]
const CLASS_NAMES: &[&str] = &[
    "FEMALE_GENITALIA_COVERED",  // 0 - bikini bottom
    "FACE_FEMALE",               // 1
    "BUTTOCKS_EXPOSED",          // 2 - explicit
    "FEMALE_BREAST_EXPOSED",     // 3 - explicit
    "FEMALE_GENITALIA_EXPOSED",  // 4 - explicit
    "MALE_BREAST_EXPOSED",       // 5
    "ANUS_EXPOSED",              // 6 - explicit
    "FEET_EXPOSED",              // 7
    "BELLY_COVERED",             // 8
    "FEET_COVERED",              // 9
    "ARMPITS_COVERED",           // 10
    "ARMPITS_EXPOSED",           // 11
    "FACE_MALE",                 // 12
    "BELLY_EXPOSED",             // 13 - suggestive (midriff)
    "MALE_GENITALIA_EXPOSED",    // 14 - explicit
    "ANUS_COVERED",              // 15
    "FEMALE_BREAST_COVERED",     // 16 - bikini top
    "BUTTOCKS_COVERED",          // 17 - bikini bottom
];

#[derive(Debug, Clone)]
pub struct NsfwResult {
    pub is_nsfw: bool,
    pub confidence: f32,
    pub category: NsfwCategory,
    pub class_name: String,
}

#[derive(Debug, Clone)]
pub enum NsfwCategory {
    Safe,
    Suggestive,  // Bikini, swimwear
    Nudity,      // Exposed body parts
    Explicit,    // Genitalia
}

pub struct NsfwDetector {
    session: Session,
}

impl NsfwDetector {
    pub fn new(model_path: &str) -> Result<Self, String> {
        if !Path::new(model_path).exists() {
            return Err(format!("Model file not found: {}", model_path));
        }

        // Initialize ONNX Runtime
        ort::init().commit();

        let session = Session::builder()
            .map_err(|e| format!("Session builder error: {}", e))?
            .with_intra_threads(2)
            .map_err(|e| format!("Thread config error: {}", e))?
            .commit_from_file(model_path)
            .map_err(|e| format!("Model load error: {}", e))?;

        Ok(Self { session })
    }

    pub fn detect(&mut self, bgra_data: &[u8], width: u32, height: u32) -> Result<NsfwResult, String> {
        // Preprocess: resize to 320x320, pad to square, normalize to [0,1]
        let input_data = self.preprocess(bgra_data, width, height)?;
        
        // Create tensor from preprocessed data
        let input_tensor = Tensor::from_array(([1usize, 3, INPUT_SIZE, INPUT_SIZE], input_data.into_boxed_slice()))
            .map_err(|e| format!("Tensor creation error: {}", e))?;

        // Run inference
        let outputs = self.session.run(ort::inputs!["images" => input_tensor])
            .map_err(|e| format!("Inference error: {}", e))?;

        // Parse YOLO-style output
        let output = outputs.get("output0")
            .ok_or("No output0 in model")?;
        
        let (shape, data) = output.try_extract_tensor::<f32>()
            .map_err(|e| format!("Output extract error: {}", e))?;
        
        // Convert shape from i64 to usize and copy data to owned vec
        let shape_usize: Vec<usize> = shape.iter().map(|&x| x as usize).collect();
        let data_owned: Vec<f32> = data.to_vec();
        
        // Drop outputs before calling postprocess
        drop(outputs);
        
        self.postprocess(&data_owned, &shape_usize)
    }

    fn preprocess(&self, bgra_data: &[u8], width: u32, height: u32) -> Result<Vec<f32>, String> {
        // Pad to square
        let max_dim = width.max(height) as usize;
        
        // Create padded image (black background)
        let mut padded = vec![0u8; max_dim * max_dim * 4];
        
        // Copy original image to padded
        for y in 0..height as usize {
            for x in 0..width as usize {
                let src_idx = (y * width as usize + x) * 4;
                let dst_idx = (y * max_dim + x) * 4;
                if src_idx + 4 <= bgra_data.len() && dst_idx + 4 <= padded.len() {
                    padded[dst_idx..dst_idx + 4].copy_from_slice(&bgra_data[src_idx..src_idx + 4]);
                }
            }
        }

        // Resize to 320x320 and convert to CHW format
        let mut resized = vec![0f32; 3 * INPUT_SIZE * INPUT_SIZE];
        let scale = max_dim as f32 / INPUT_SIZE as f32;

        for y in 0..INPUT_SIZE {
            for x in 0..INPUT_SIZE {
                let src_x = (x as f32 * scale) as usize;
                let src_y = (y as f32 * scale) as usize;
                let src_idx = (src_y * max_dim + src_x) * 4;

                if src_idx + 2 < padded.len() {
                    // BGRA to RGB (CHW layout), normalize to [0,1]
                    resized[0 * INPUT_SIZE * INPUT_SIZE + y * INPUT_SIZE + x] = padded[src_idx + 2] as f32 / 255.0; // R
                    resized[1 * INPUT_SIZE * INPUT_SIZE + y * INPUT_SIZE + x] = padded[src_idx + 1] as f32 / 255.0; // G
                    resized[2 * INPUT_SIZE * INPUT_SIZE + y * INPUT_SIZE + x] = padded[src_idx + 0] as f32 / 255.0; // B
                }
            }
        }

        Ok(resized)
    }

    fn postprocess(&self, data: &[f32], shape: &[usize]) -> Result<NsfwResult, String> {
        // NudeNet 320n output shape: [1, 22, N] where N = number of detection anchors
        // 22 = 4 (bbox: x, y, w, h) + 18 (class scores)
        
        let frame_num = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
        
        if shape.len() < 3 {
            return Ok(NsfwResult { 
                is_nsfw: false, 
                confidence: 0.0, 
                category: NsfwCategory::Safe,
                class_name: "none".to_string(),
            });
        }

        let num_features = shape[1]; // 22
        let num_detections = shape[2];

        // Track best detection for each category
        let mut best_explicit_conf = 0.0f32;
        let mut best_explicit_class: Option<usize> = None;
        let mut best_suggestive_conf = 0.0f32;
        let mut best_suggestive_class: Option<usize> = None;
        
        // Also track all high-confidence detections for debug
        let mut detections: Vec<(usize, f32)> = Vec::new();

        // Data is in [1, 22, N] format
        for det_idx in 0..num_detections {
            // Check ALL class scores for this detection
            for cls_idx in 0..18 {
                let feat_idx = 4 + cls_idx; // 4 bbox values + class index
                if feat_idx < num_features {
                    let data_idx = feat_idx * num_detections + det_idx;
                    if data_idx < data.len() {
                        let conf = data[data_idx];
                        
                        // Track high-confidence detections for debug
                        if conf > 0.3 {
                            detections.push((cls_idx, conf));
                        }
                        
                        // Check explicit classes
                        if EXPLICIT_CLASSES.contains(&cls_idx) && conf > best_explicit_conf {
                            best_explicit_conf = conf;
                            best_explicit_class = Some(cls_idx);
                        }
                        
                        // Check suggestive classes
                        if SUGGESTIVE_CLASSES.contains(&cls_idx) && conf > best_suggestive_conf {
                            best_suggestive_conf = conf;
                            best_suggestive_class = Some(cls_idx);
                        }
                    }
                }
            }
        }

        // Log every 10th frame with detections
        if frame_num % 10 == 0 && !detections.is_empty() {
            // Aggregate by class
            let mut class_max: std::collections::HashMap<usize, f32> = std::collections::HashMap::new();
            for (cls, conf) in &detections {
                let entry = class_max.entry(*cls).or_insert(0.0);
                if *conf > *entry {
                    *entry = *conf;
                }
            }
            let mut sorted: Vec<_> = class_max.iter().collect();
            sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
            let top5: Vec<String> = sorted.iter().take(5)
                .map(|(cls, conf)| format!("{}:{:.0}%", CLASS_NAMES.get(**cls).unwrap_or(&"?"), *conf * 100.0))
                .collect();
            if !top5.is_empty() {
                println!("    [Frame {}] Top detections: {}", frame_num, top5.join(", "));
            }
        }

        // Priority: Explicit > Suggestive
        if best_explicit_conf > EXPLICIT_THRESHOLD {
            if let Some(cls) = best_explicit_class {
                let category = match cls {
                    4 | 14 | 6 => NsfwCategory::Explicit,
                    _ => NsfwCategory::Nudity,
                };
                let class_name = CLASS_NAMES.get(cls).unwrap_or(&"unknown").to_string();
                println!("🚨 NSFW DETECTED: {} ({:.0}%)", class_name, best_explicit_conf * 100.0);
                return Ok(NsfwResult {
                    is_nsfw: true,
                    confidence: best_explicit_conf,
                    category,
                    class_name,
                });
            }
        }

        // Check suggestive content (bikini)
        if best_suggestive_conf > SUGGESTIVE_THRESHOLD {
            if let Some(cls) = best_suggestive_class {
                let class_name = CLASS_NAMES.get(cls).unwrap_or(&"unknown").to_string();
                println!("⚠️ SUGGESTIVE DETECTED: {} ({:.0}%)", class_name, best_suggestive_conf * 100.0);
                return Ok(NsfwResult {
                    is_nsfw: true,
                    confidence: best_suggestive_conf,
                    category: NsfwCategory::Suggestive,
                    class_name,
                });
            }
        }

        Ok(NsfwResult {
            is_nsfw: false,
            confidence: 0.0,
            category: NsfwCategory::Safe,
            class_name: "none".to_string(),
        })
    }
}

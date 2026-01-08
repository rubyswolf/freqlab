//! Live audio input capture using cpal
//!
//! Captures audio from an input device (microphone, audio interface) and
//! provides samples to the audio engine via a lock-free ring buffer.

use cpal::traits::{DeviceTrait, StreamTrait};
use parking_lot::{Mutex, RwLock};
use ringbuf::{traits::*, HeapRb};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use super::buffer::StereoSample;
use super::device::{get_input_device, get_native_input_config};

/// Helper to store f32 in AtomicU32
#[inline]
fn f32_to_u32(f: f32) -> u32 {
    f.to_bits()
}

/// Helper to load f32 from AtomicU32
#[inline]
fn u32_to_f32(u: u32) -> f32 {
    f32::from_bits(u)
}

/// Shared state between input stream and readers
struct InputSharedState {
    /// Ring buffer consumer (read side) - wrapped in Mutex for thread-safe access
    /// Mutex is used instead of RwLock because ringbuf's consumer contains Cell<usize>
    consumer: Mutex<ringbuf::HeapCons<StereoSample>>,
    /// Input levels for metering (atomic for lock-free access)
    input_level_left: AtomicU32,
    input_level_right: AtomicU32,
    /// Whether the input stream is active
    is_active: AtomicBool,
    /// Error state
    error: RwLock<Option<String>>,
    /// Number of input channels (1 = mono, 2 = stereo)
    input_channels: u16,
    /// Actual sample rate of input device
    actual_sample_rate: u32,
}

/// Handle to control and read from the input capture
#[derive(Clone)]
pub struct InputCaptureHandle {
    shared: Arc<InputSharedState>,
}

impl InputCaptureHandle {
    /// Read a stereo sample from the input buffer
    /// Returns silence if buffer is empty
    pub fn read_sample(&self) -> StereoSample {
        if let Some(sample) = self.shared.consumer.lock().try_pop() {
            sample
        } else {
            StereoSample::silence()
        }
    }

    /// Read multiple samples into the output slice
    /// Returns the number of samples actually read
    pub fn read_samples(&self, output: &mut [StereoSample]) -> usize {
        self.shared.consumer.lock().pop_slice(output)
    }

    /// Get current input levels (0.0 - 1.0)
    pub fn get_input_levels(&self) -> (f32, f32) {
        let left = u32_to_f32(self.shared.input_level_left.load(Ordering::Relaxed));
        let right = u32_to_f32(self.shared.input_level_right.load(Ordering::Relaxed));
        (left, right)
    }

    /// Check if the input stream is active
    pub fn is_active(&self) -> bool {
        self.shared.is_active.load(Ordering::SeqCst)
    }

    /// Get any error that occurred
    pub fn get_error(&self) -> Option<String> {
        self.shared.error.read().clone()
    }

    /// Clear the buffer (call when starting/unpausing to avoid stale data)
    pub fn clear_buffer(&self) {
        let mut consumer = self.shared.consumer.lock();
        // Drain all samples
        while consumer.try_pop().is_some() {}
    }

    /// Get the actual sample rate of the input device
    pub fn sample_rate(&self) -> u32 {
        self.shared.actual_sample_rate
    }

    /// Get the number of available samples in the buffer
    pub fn available_samples(&self) -> usize {
        self.shared.consumer.lock().occupied_len()
    }
}

/// Input capture stream
/// Holds the cpal stream and shared state
pub struct InputCapture {
    _stream: cpal::Stream,
    handle: InputCaptureHandle,
    device_name: String,
}

impl InputCapture {
    /// Create a new input capture from the specified device
    ///
    /// # Arguments
    /// * `device_name` - Name of the input device, or None for system default
    /// * `buffer_size_samples` - Size of the ring buffer in stereo samples
    ///
    /// Note: Always uses the device's native sample rate to avoid CoreAudio conflicts.
    /// The engine will use a resampler if the output sample rate differs.
    pub fn new(
        device_name: Option<&str>,
        buffer_size_samples: usize,
    ) -> Result<Self, String> {
        let device = get_input_device(device_name)?;
        let device_name_str = device.name().unwrap_or_else(|_| "Unknown".to_string());

        log::info!("Creating input capture for device: {}", device_name_str);

        // Always use the device's native sample rate to avoid CoreAudio conflicts
        // The engine will resample if needed
        let stream_config = get_native_input_config(&device)?;
        let actual_sample_rate = stream_config.sample_rate.0;
        let channels = stream_config.channels;

        log::info!(
            "Input stream config: {} Hz, {} channels",
            actual_sample_rate,
            channels
        );

        // Create ring buffer - size should be enough to handle timing jitter
        // Use 3x the typical buffer size for safety
        let rb = HeapRb::new(buffer_size_samples);
        let (mut producer, consumer) = rb.split();

        // Create shared state
        let shared = Arc::new(InputSharedState {
            consumer: Mutex::new(consumer),
            input_level_left: AtomicU32::new(f32_to_u32(0.0)),
            input_level_right: AtomicU32::new(f32_to_u32(0.0)),
            is_active: AtomicBool::new(true),
            error: RwLock::new(None),
            input_channels: channels,
            actual_sample_rate,
        });

        let shared_clone = Arc::clone(&shared);
        let level_smoothing = 0.15f32; // Slightly faster response for input metering

        // Build the input stream
        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let channels = shared_clone.input_channels as usize;
                    let mut peak_left = 0.0f32;
                    let mut peak_right = 0.0f32;

                    // Process input samples
                    for chunk in data.chunks(channels) {
                        let (left, right) = if channels == 1 {
                            // Mono input - duplicate to both channels
                            let mono = chunk[0];
                            (mono, mono)
                        } else {
                            // Stereo input
                            (chunk[0], chunk.get(1).copied().unwrap_or(chunk[0]))
                        };

                        // Track peak levels
                        peak_left = peak_left.max(left.abs());
                        peak_right = peak_right.max(right.abs());

                        // Push to ring buffer (drop samples if buffer is full)
                        let _ = producer.try_push(StereoSample::new(left, right));
                    }

                    // Update input levels with smoothing
                    {
                        let current = u32_to_f32(shared_clone.input_level_left.load(Ordering::Relaxed));
                        let new_level = current * (1.0 - level_smoothing) + peak_left * level_smoothing;
                        shared_clone.input_level_left.store(f32_to_u32(new_level), Ordering::Relaxed);
                    }
                    {
                        let current = u32_to_f32(shared_clone.input_level_right.load(Ordering::Relaxed));
                        let new_level = current * (1.0 - level_smoothing) + peak_right * level_smoothing;
                        shared_clone.input_level_right.store(f32_to_u32(new_level), Ordering::Relaxed);
                    }
                },
                move |err| {
                    log::error!("Input stream error: {}", err);
                    // Note: We can't easily update shared state here since we moved shared_clone
                    // The error will manifest as buffer underruns
                },
                None, // No timeout
            )
            .map_err(|e| format!("Failed to build input stream: {}", e))?;

        // Start the stream
        stream
            .play()
            .map_err(|e| format!("Failed to start input stream: {}", e))?;

        log::info!("Input capture started successfully");

        let handle = InputCaptureHandle { shared };

        Ok(Self {
            _stream: stream,
            handle,
            device_name: device_name_str,
        })
    }

    /// Get a handle to read from this input capture
    pub fn handle(&self) -> InputCaptureHandle {
        self.handle.clone()
    }

    /// Get the device name
    pub fn device_name(&self) -> &str {
        &self.device_name
    }
}

impl Drop for InputCapture {
    fn drop(&mut self) {
        log::info!("Input capture stopped: {}", self.device_name);
        self.handle.shared.is_active.store(false, Ordering::SeqCst);
    }
}

// ============================================================================
// Global Input Capture Management
// ============================================================================

/// Global handle for easy access from engine
/// Note: We only store the handle globally, not the InputCapture itself,
/// because cpal::Stream is not Send+Sync. The InputCapture is leaked
/// intentionally to keep the stream alive for the app's lifetime.
static INPUT_HANDLE: once_cell::sync::OnceCell<RwLock<Option<InputCaptureHandle>>> =
    once_cell::sync::OnceCell::new();

/// Start live input capture
/// Uses the device's native sample rate to avoid CoreAudio conflicts
pub fn start_input_capture(device_name: Option<&str>) -> Result<InputCaptureHandle, String> {
    // Stop any existing capture first
    stop_input_capture();

    // Create buffer sized for ~100ms of audio
    // Use a reasonable estimate for buffer sizing (actual rate is determined by device)
    let buffer_size = 4800 * 3; // ~100ms at 48kHz, 3x for safety margin

    let capture = InputCapture::new(device_name, buffer_size)?;
    let handle = capture.handle();

    // Store handle globally
    let handle_cell = INPUT_HANDLE.get_or_init(|| RwLock::new(None));
    *handle_cell.write() = Some(handle.clone());

    // Leak the InputCapture to keep the stream alive
    // This is intentional - the stream should live for the app's lifetime
    // Same pattern used by the main audio engine
    std::mem::forget(capture);

    log::info!("Input capture started and stream leaked for lifetime management");

    Ok(handle)
}

/// Stop live input capture
pub fn stop_input_capture() {
    if let Some(cell) = INPUT_HANDLE.get() {
        if let Some(handle) = cell.write().take() {
            // Mark as inactive
            handle.shared.is_active.store(false, Ordering::SeqCst);
            log::info!("Input capture stopped (handle dropped)");
        }
    }
    // Note: The actual cpal::Stream was leaked and will be cleaned up on process exit
    // This is the same pattern used by the main audio engine
}

/// Get the global input capture handle (if active)
pub fn get_input_handle() -> Option<InputCaptureHandle> {
    INPUT_HANDLE
        .get()
        .and_then(|cell| cell.read().clone())
}

/// Check if input capture is active
pub fn is_input_capture_active() -> bool {
    get_input_handle().map(|h| h.is_active()).unwrap_or(false)
}

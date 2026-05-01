use btleplug::api::WriteType;
use btleplug::platform::Peripheral;
use futures::future::ok;
use futures::StreamExt;
use lazy_static::lazy_static;
use lsl::Pushable;
use lsl::StreamOutlet;
use lsl::{ChannelFormat, StreamInfo};
use serde_json::json;
use serialport;
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{self, AppHandle, Emitter};
use tokio::sync::mpsc;
use tungstenite::connect;
use tungstenite::protocol::Message;
use url::Url;

lazy_static! {
    static ref BAUDRATE: Arc<Mutex<u32>> = Arc::new(Mutex::new(230400));
    static ref PACKET_SIZE: Arc<Mutex<usize>> = Arc::new(Mutex::new(16));
    static ref CHANNELS: Arc<Mutex<usize>> = Arc::new(Mutex::new(6));
    static ref BITS: Arc<Mutex<String>> = Arc::new(Mutex::new("10".into()));
    static ref SAMPLE_RATE: Arc<Mutex<f64>> = Arc::new(Mutex::new(500.0));

    // Dynamic BLE config derived from device name
    static ref BLE_CHANNELS: Arc<Mutex<usize>> = Arc::new(Mutex::new(3));
    static ref BLE_SAMPLE_LEN: Arc<Mutex<usize>> = Arc::new(Mutex::new(7));
    static ref BLE_OLD_FIRMWARE: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

use std::collections::VecDeque;
use tauri::Manager;

// ─── Serial / Arduino detection ──────────────────────────────────────────────

#[tauri::command]
async fn detect_arduino() -> Result<String, String> {
    tokio::task::spawn_blocking(|| detect_arduino_internal())
        .await
        .map_err(|e| format!("Task panicked: {:?}", e))?
}

fn detect_arduino_internal() -> Result<String, String> {
    loop {
        let ports = serialport::available_ports().expect("No ports found!");

        for port_info in ports {
            let port_name = port_info.port_name;
            println!("Attempting to connect to port: {}", port_name);

            if port_name.contains("BLTH")
                || port_name.contains("Bluetooth")
                || port_name.contains("console")
            {
                continue;
            }
            if let serialport::SerialPortType::UsbPort(info) = port_info.port_type {
                if info.vid == 6790 || matches!(info.pid, 67 | 579 | 29987 | 66 | 24577) {
                    *BAUDRATE.lock().unwrap() = 115200;
                    *SAMPLE_RATE.lock().unwrap() = 250.0;
                }
            }

            match serialport::new(&port_name, *BAUDRATE.lock().unwrap())
                .timeout(Duration::from_secs(3))
                .open()
            {
                Ok(mut port) => {
                    thread::sleep(Duration::from_secs(3));
                    let command = b"WHORU\n";

                    if let Err(e) = port.write_all(command) {
                        println!("Failed to write to port: {}. Error: {:?}", port_name, e);
                        continue;
                    }
                    port.flush().expect("Failed to flush port");

                    let mut buffer: Vec<u8> = vec![0; 1024];
                    let mut response = String::new();
                    let start_time = Instant::now();
                    let timeout = Duration::from_secs(10);

                    while start_time.elapsed() < timeout {
                        match port.read(&mut buffer) {
                            Ok(size) => {
                                if size > 0 {
                                    response.push_str(&String::from_utf8_lossy(&buffer[..size]));
                                    if response.contains("UNO-R4")
                                        || response.contains("UNO-R3")
                                        || response.contains("GIGA-R1")
                                        || response.contains("RPI-PICO-RP2040")
                                        || response.contains("UNO-CLONE")
                                        || response.contains("NANO-CLONE")
                                        || response.contains("MEGA-2560-R3")
                                        || response.contains("MEGA-2560-CLONE")
                                        || response.contains("GENUINO-UNO")
                                        || response.contains("NANO-CLASSIC")
                                        || response.contains("STM32G4-CORE-BOARD")
                                        || response.contains("STM32F4-BLACK-PILL")
                                        || response.contains("NPG-LITE")
                                    {
                                        if response.contains("UNO-R3")
                                            || response.contains("UNO-CLONE")
                                            || response.contains("NANO-CLONE")
                                            || response.contains("MEGA-2560-R3")
                                            || response.contains("MEGA-2560-CLONE")
                                            || response.contains("GENUINO-UNO")
                                            || response.contains("NANO-CLASSIC")
                                        {
                                            *BITS.lock().unwrap() = "10".into();
                                        } else if response.contains("GIGA-R1") {
                                            *BITS.lock().unwrap() = "16".into();
                                        } else if response.contains("RPI-PICO-RP2040")
                                            || response.contains("STM32G4-CORE-BOARD")
                                            || response.contains("STM32F4-BLACK-PILL")
                                            || response.contains("NPG-LITE")
                                        {
                                            *BITS.lock().unwrap() = "12".into();
                                        } else if response.contains("UNO-R4") {
                                            *BITS.lock().unwrap() = "14".into();
                                        }
                                        if response.contains("NANO-CLONE")
                                            || response.contains("NANO-CLASSIC")
                                            || response.contains("STM32F4-BLACK-PILL")
                                        {
                                            *PACKET_SIZE.lock().unwrap() = 20;
                                            *CHANNELS.lock().unwrap() = 8;
                                        }
                                        if response.contains("MEGA-2560-R3")
                                            || response.contains("MEGA-2560-CLONE")
                                            || response.contains("STM32G4-CORE-BOARD")
                                        {
                                            *PACKET_SIZE.lock().unwrap() = 36;
                                            *CHANNELS.lock().unwrap() = 16;
                                        }
                                        if response.contains("RPI-PICO-RP2040")
                                            || response.contains("NPG-LITE")
                                        {
                                            *PACKET_SIZE.lock().unwrap() = 10;
                                            *CHANNELS.lock().unwrap() = 3;
                                        }
                                        println!("Valid device found on port: {}", port_name);
                                        drop(port);
                                        return Ok(port_name);
                                    }
                                }
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
                            Err(e) => {
                                println!("Failed to read from port: {}. Error: {:?}", port_name, e);
                                break;
                            }
                        }
                    }
                    println!("Final response from port {}: {}", port_name, response);
                    drop(port);
                }
                Err(e) => {
                    println!("Failed to open port: {}. Error: {:?}", port_name, e);
                }
            }
        }

        println!("No valid device found, retrying in 5 seconds...");
        thread::sleep(Duration::from_secs(5));
    }
}

// ─── Serial streaming ─────────────────────────────────────────────────────────

#[tauri::command]
async fn start_streaming(port_name: String, app_handle: AppHandle) {
    const START_BYTE_1: u8 = 0xC7;
    const START_BYTE_2: u8 = 0x7C;
    const END_BYTE: u8 = 0x01;

    let (tx, mut rx) = mpsc::channel::<Vec<i16>>(100);

    let mut info = StreamInfo::new(
        "UDL",
        "Biopotential_Signals",
        (*CHANNELS.lock().unwrap()).try_into().unwrap(),
        *SAMPLE_RATE.lock().unwrap(),
        lsl::ChannelFormat::Int16,
        "Chords",
    )
    .unwrap();

    let mut desc = info.desc();
    let mut resinfo = desc.append_child("resinfo");
    resinfo.append_child_value("resolution", &*BITS.lock().unwrap());

    println!("LSL Stream XML Description:");
    match info.to_xml() {
        Ok(xml) => println!("{}", xml),
        Err(e) => println!("Failed to get XML description: {:?}", e),
    }

    let (tx, rx) = std::sync::mpsc::channel::<Vec<i16>>();
    let outlet = Arc::new(Mutex::new(StreamOutlet::new(&info, 0, 360).unwrap()));

    tokio::task::spawn_blocking(move || loop {
        match serialport::new(&port_name, *BAUDRATE.lock().unwrap())
            .timeout(Duration::from_secs(3))
            .open()
        {
            Ok(mut port) => {
                let start_command = b"START\r\n";

                for i in 1..=3 {
                    if let Err(e) = port.write_all(start_command) {}
                    println!("Connected to device on port: {}", port_name);
                    thread::sleep(Duration::from_millis(1000));
                }

                println!("Finished sending commands.");

                let mut buffer: Vec<u8> = vec![0; 1024];
                let mut accumulated_buffer: Vec<u8> = Vec::new();

                let mut packet_count = 0;
                let mut sample_count = 0;
                let mut byte_count = 0;
                let start_time = Instant::now();
                let mut last_print_time = Instant::now();
                packet_count += 1;

                loop {
                    match port.read(&mut buffer) {
                        Ok(size) => {
                            accumulated_buffer.extend_from_slice(&buffer[..size]);
                            byte_count += size;

                            while accumulated_buffer.len() >= *PACKET_SIZE.lock().unwrap() {
                                if accumulated_buffer[0] == START_BYTE_1
                                    && accumulated_buffer[1] == START_BYTE_2
                                {
                                    if accumulated_buffer[*PACKET_SIZE.lock().unwrap() - 1]
                                        == END_BYTE
                                    {
                                        let packet = accumulated_buffer
                                            .drain(..*PACKET_SIZE.lock().unwrap())
                                            .collect::<Vec<u8>>();
                                        sample_count += 1;
                                        let data: Vec<i16> = (0..*CHANNELS.lock().unwrap())
                                            .map(|i| {
                                                let idx = 3 + (i * 2);
                                                let high = packet[idx] as i16;
                                                let low = packet[idx + 1] as i16;
                                                (high << 8) | low
                                            })
                                            .collect();
                                        if tx.send(data).is_err() {
                                            println!("Failed to send data to the main thread.");
                                            break;
                                        }
                                    } else {
                                        accumulated_buffer.drain(..1);
                                    }
                                } else {
                                    accumulated_buffer.drain(..1);
                                }
                            }

                            if last_print_time.elapsed() >= Duration::from_secs(1) {
                                let elapsed = start_time.elapsed().as_secs_f32();
                                let samples_per_second =
                                    format!("{:.2}", sample_count as f32);

                                let _ = app_handle.emit("connection", "Connected ");
                                if elapsed > 2.0 {
                                    let _ = app_handle.emit("samplerate", samples_per_second);
                                }
                                let _ = app_handle.emit("lsl", "uidserial007");
                                sample_count = 0;
                                last_print_time = Instant::now();
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                            println!("Read timed out, retrying...");
                            continue;
                        }
                        Err(e) => {
                            println!("Error receiving data: {:?}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to connect to device on {}: {}", port_name, e);
                break;
            }
        }

        println!("Device disconnected, checking for new devices...");
        thread::sleep(Duration::from_secs(5));
    });

    while let Ok(data) = rx.recv() {
        if let Ok(outlet) = outlet.lock() {
            outlet.push_sample(&data).unwrap_or_else(|e| {
                println!("Failed to push data to LSL: {:?}", e);
            });
        }
    }
}

fn calculate_rate(data_size: usize, elapsed_time: f64) -> f64 {
    data_size as f64 / elapsed_time
}

// ─── WiFi streaming ───────────────────────────────────────────────────────────

#[tauri::command]
async fn start_wifistreaming(app_handle: AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        let stream_name = "NPG-Lite";
        let mut info = StreamInfo::new(
            stream_name,
            "EXG",
            3,
            500.0,
            ChannelFormat::Float32,
            "uidwifi007",
        )
        .expect("Failed to create StreamInfo");
        let mut desc = info.desc();
        let mut resinfo = desc.append_child("resinfo");
        resinfo.append_child_value("resolution", "12");

        let outlet = StreamOutlet::new(&info, 0, 360).expect("Failed to create StreamOutlet");

        let ws_url = "ws://multi-emg.local:81";
        let (mut socket, _) =
            connect(Url::parse(ws_url).expect("Failed to parse URL")).expect("WebSocket failed");
        println!("{} WebSocket connected!", stream_name);
        let _ = app_handle.emit("connection", "Connected");
        let mut block_size = 13;
        let mut packet_size = 0;
        let mut data_size = 0;
        let mut sample_size = 0;
        let mut previous_sample_number: Option<u8> = None;
        let mut previous_data = vec![];
        let mut sample_count = 0;
        let mut samplelost = 0;
        let mut start_time = Instant::now();
        const BUFFER_SIZE: usize = 20;

        let mut buffer: VecDeque<f64> = VecDeque::with_capacity(BUFFER_SIZE);

        loop {
            match socket.read_message() {
                Ok(Message::Binary(data)) => {
                    data_size += data.len();
                    let elapsed_time = start_time.elapsed().as_secs_f64();

                    if elapsed_time >= 1.0 {
                        let samples_per_second = calculate_rate(sample_size, elapsed_time);
                        let refresh_rate = calculate_rate(packet_size, elapsed_time);
                        let bytes_per_second = calculate_rate(data_size, elapsed_time);

                        let sps = samples_per_second.floor();

                        if buffer.len() == BUFFER_SIZE {
                            buffer.pop_front();
                        }
                        buffer.push_back(sps);

                        let max_sps: f64 = *buffer
                            .iter()
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap_or(&sps);

                        println!(
                            "{} FPS : {} SPS : {} BPS (AVG SPS: {:.2})",
                            refresh_rate.ceil(),
                            sps,
                            bytes_per_second.ceil(),
                            max_sps
                        );

                        let _ = app_handle.emit("samplerate", max_sps);

                        packet_size = 0;
                        sample_size = 0;
                        data_size = 0;
                        start_time = Instant::now();
                    }
                    packet_size += 1;

                    for block_location in (0..data.len()).step_by(block_size) {
                        sample_size += 1;
                        let block = &data[block_location..block_location + block_size];
                        let sample_number = block[0];
                        let mut channel_data = vec![];

                        for channel in 0..3 {
                            let offset = 1 + (channel * 2);
                            let sample = i16::from_be_bytes([block[offset], block[offset + 1]]);
                            channel_data.push(sample as f32);
                        }
                        sample_count += 1;
                        if let Some(prev) = previous_sample_number {
                            if sample_number.wrapping_sub(prev) > 1 {
                                println!("Error: Sample Lost");
                                samplelost += 1;
                                let _ = app_handle.emit("samplelost", samplelost);
                                break;
                            } else if sample_number == prev {
                                println!("Error: Duplicate Sample");
                                break;
                            }
                        }

                        previous_sample_number = Some(sample_number);
                        previous_data = channel_data.clone();

                        if let Err(err) = outlet.push_sample(&channel_data) {
                            eprintln!("Failed to push sample: {}", err);
                        }

                        let _ = app_handle.emit("connection", "Connected");

                        if sample_count % 100 == 0 {
                            let _ = app_handle.emit("lsl", "uidwifi007");
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("WebSocket error: {:?}", e);
                    break;
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    });
}

// ─── BLE ─────────────────────────────────────────────────────────────────────

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager as BtleManager;

struct SafeOutlet(Option<StreamOutlet>);
unsafe impl Send for SafeOutlet {}
unsafe impl Sync for SafeOutlet {}

lazy_static! {
    static ref BLE_OUTLET: Arc<Mutex<SafeOutlet>> = Arc::new(Mutex::new(SafeOutlet(None)));
    static ref BLE_SAMPLE_COUNTER: AtomicU8 = AtomicU8::new(0);
    static ref BLE_CONNECTED: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref CONNECTED_PERIPHERAL: Arc<Mutex<Option<Peripheral>>> =
        Arc::new(Mutex::new(None));
}

/// Derive BLE channel/sample-length config from the advertised device name,
/// mirroring the JS logic:
///   "3CH" → 3 ch, 7 bytes/sample, new firmware
///   "6CH" → 6 ch, 13 bytes/sample, new firmware
///   else  → 3 ch, 7 bytes/sample, old firmware
fn apply_ble_device_config(name: &str) {
    if name.contains("3CH") {
        *BLE_CHANNELS.lock().unwrap() = 3;
        *BLE_SAMPLE_LEN.lock().unwrap() = 7; // 1 counter + 3*2
        *BLE_OLD_FIRMWARE.lock().unwrap() = false;
        println!("[CONFIG] 3CH device: channels=3, sample_len=7, new firmware");
    } else if name.contains("6CH") {
        *BLE_CHANNELS.lock().unwrap() = 6;
        *BLE_SAMPLE_LEN.lock().unwrap() = 13; // 1 counter + 6*2
        *BLE_OLD_FIRMWARE.lock().unwrap() = false;
        println!("[CONFIG] 6CH device: channels=6, sample_len=13, new firmware");
    } else {
        *BLE_CHANNELS.lock().unwrap() = 3;
        *BLE_SAMPLE_LEN.lock().unwrap() = 7;
        *BLE_OLD_FIRMWARE.lock().unwrap() = true;
        println!("[CONFIG] Unknown/old device \"{}\": channels=3, sample_len=7, old firmware", name);
    }
}

/// Create (or recreate) the LSL outlet using the currently configured channel count.
fn create_ble_outlet() -> Result<(), String> {
    let channels = *BLE_CHANNELS.lock().unwrap();

    let mut info = StreamInfo::new(
        "NPG-Lite",
        "EXG",
        channels.try_into().unwrap(),
        500.0,
        ChannelFormat::Float32,
        "uidbluetooth007",
    )
    .map_err(|e| e.to_string())?;

    let mut desc = info.desc();
    let mut resinfo = desc.append_child("resinfo");
    resinfo.append_child_value("resolution", "12");

    match info.to_xml() {
        Ok(xml) => println!("✅ Final LSL StreamInfo:\n{}", xml),
        Err(e) => println!("❌ XML error: {}", e),
    }

    let outlet = StreamOutlet::new(&info, 0, 360).map_err(|e| e.to_string())?;
    *BLE_OUTLET.lock().unwrap() = SafeOutlet(Some(outlet));
    Ok(())
}

fn close_ble_outlet() {
    *BLE_OUTLET.lock().unwrap() = SafeOutlet(None);
    BLE_SAMPLE_COUNTER.store(0, Ordering::Relaxed);
    *BLE_CONNECTED.lock().unwrap() = false;
}

/// Decode one raw BLE sample.
/// Expected layout: [counter, ch0_hi, ch0_lo, ch1_hi, ch1_lo, …]
/// Length must equal 1 + channels * 2.
fn process_ble_sample(sample: &[u8], app_handle: AppHandle) -> Result<Vec<f32>, String> {
    let channels = *BLE_CHANNELS.lock().unwrap();
    let expected_len = 1 + channels * 2;

    if sample.len() != expected_len {
        return Err(format!(
            "Invalid sample length: got {}, expected {} ({} channels)",
            sample.len(),
            expected_len,
            channels
        ));
    }

    let mut samplelost = 0u32;
    let sample_counter = sample[0];
    let prev = BLE_SAMPLE_COUNTER.load(Ordering::Relaxed);
    let expected_counter = prev.wrapping_add(1);

    if sample_counter != expected_counter {
        samplelost += 1;
        println!(
            "Sample counter discontinuity: expected {}, got {}",
            expected_counter, sample_counter
        );
    }
    BLE_SAMPLE_COUNTER.store(sample_counter, Ordering::Relaxed);
    let _ = app_handle.emit("samplelost", samplelost);

    let decoded: Vec<f32> = (0..channels)
        .map(|i| {
            let hi = sample[1 + i * 2];
            let lo = sample[2 + i * 2];
            i16::from_be_bytes([hi, lo]) as f32
        })
        .collect();

    Ok(decoded)
}

// ─── BLE scan ─────────────────────────────────────────────────────────────────

#[tauri::command]
async fn scan_ble_devices(app_handle: AppHandle) -> Result<(), String> {
    let manager = BtleManager::new()
        .await
        .map_err(|e| format!("Manager creation failed: {}", e))?;

    let adapter = manager
        .adapters()
        .await
        .map_err(|e| format!("Failed to get adapters: {}", e))?
        .into_iter()
        .next()
        .ok_or("No Bluetooth adapters found".to_string())?;

    println!(
        "Using adapter: {}",
        adapter.adapter_info().await.map_err(|e| e.to_string())?
    );

    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| format!("Failed to start scan: {}", e))?;

    println!("Scanning for BLE devices...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    let peripherals = adapter
        .peripherals()
        .await
        .map_err(|e| format!("Failed to get peripherals: {}", e))?;

    if peripherals.is_empty() {
        println!("No BLE devices found");
        return Err("No BLE devices found".to_string());
    }

    let mut devices = Vec::new();
    for peripheral in peripherals {
        match peripheral.properties().await {
            Ok(Some(props)) => {
                if let Some(name) = &props.local_name {
                    if name.to_lowercase().contains("npg") {
                        println!("Found matching device: {} ({})", name, peripheral.id());
                        devices.push(json!({
                            "name": name,
                            "id": peripheral.id().to_string(),
                            "rssi": props.rssi,
                            "connected": peripheral.is_connected().await.unwrap_or(false)
                        }));
                    }
                }
            }
            Ok(None) => println!("Device with no properties"),
            Err(e) => println!("Error getting properties: {}", e),
        }
    }

    if devices.is_empty() {
        println!("No BLE devices with name containing 'npg' found");
    }

    app_handle
        .emit("bleDevices", devices)
        .map_err(|e| format!("Failed to emit devices: {}", e))?;

    Ok(())
}

// ─── BLE connect ──────────────────────────────────────────────────────────────

#[tauri::command]
async fn connect_to_ble(device_id: String, app_handle: AppHandle) -> Result<String, String> {
    println!("[CONNECT] Starting connection to device: {}", device_id);
    close_ble_outlet();

    const BUFFER_SIZE: usize = 20;
    let mut buffer: VecDeque<f64> = VecDeque::with_capacity(BUFFER_SIZE);

    let manager = match BtleManager::new().await {
        Ok(m) => { println!("[MANAGER] Bluetooth manager initialized"); m }
        Err(e) => {
            println!("[ERROR] Manager creation failed: {}", e);
            return Err(format!("Bluetooth initialization failed: {}", e));
        }
    };

    let adapters = match manager.adapters().await {
        Ok(a) => { println!("[ADAPTERS] Found {} adapter(s)", a.len()); a }
        Err(e) => {
            println!("[ERROR] Failed to get adapters: {}", e);
            return Err(format!("Adapter discovery failed: {}", e));
        }
    };

    for adapter in adapters {
        let adapter_info = match adapter.adapter_info().await {
            Ok(info) => { println!("[ADAPTER] Adapter info: {}", info); info }
            Err(e) => { println!("[WARN] Failed to get adapter info: {}", e); continue; }
        };

        let (is_windows, is_linux_hci) = {
            let info_lower = adapter_info.to_lowercase();
            (
                info_lower.contains("winrt") || info_lower.contains("windows"),
                info_lower.contains("hci"),
            )
        };

        println!(
            "[PLATFORM] Detected - Windows: {}, Linux HCI: {}",
            is_windows, is_linux_hci
        );

        if is_windows {
            println!("[WINDOWS] Starting Windows-specific pairing process...");
            use std::process::Command;
            let win_device_id = device_id.replace(":", "");
            let _ = Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!(
                        "Get-PnpDevice -InstanceId 'BTHENUM\\DEV_{}' | Where-Object {{ $_.Status -eq 'OK' }}",
                        win_device_id
                    ),
                ])
                .output();

            println!("[WINDOWS] Starting fresh scan...");
            if let Err(e) = adapter.start_scan(ScanFilter::default()).await {
                println!("[WARN] Scan failed: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        if !is_windows && !is_linux_hci {
            adapter
                .start_scan(ScanFilter::default())
                .await
                .expect("Failed to start scan");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        let peripherals = match adapter.peripherals().await {
            Ok(p) => {
                println!("[PERIPHERALS] Found {} device(s)", p.len());
                println!("[DEBUG] Listing all peripherals:");
                for (i, p) in p.iter().enumerate() {
                    println!(
                        "  {}. ID: {}, Connected: {}",
                        i + 1,
                        p.id(),
                        p.is_connected().await.unwrap_or(false)
                    );
                }
                p
            }
            Err(e) => {
                println!("[ERROR] Peripheral discovery failed: {}", e);
                continue;
            }
        };

        for peripheral in peripherals {
            let peripheral_id = peripheral.id().to_string();
            let peripheral_props = peripheral.properties().await.unwrap_or_default();

            println!("[CHECK] Checking peripheral: {}", peripheral_id);

            let is_match = if is_windows {
                let clean_peripheral_id = peripheral_id
                    .replace("BTHENUM\\", "")
                    .replace("DEV_", "")
                    .replace(":", "")
                    .to_lowercase();
                let clean_target_id = device_id.replace(":", "").to_lowercase();
                println!(
                    "[COMPARE] Windows: {} contains {}? {}",
                    clean_peripheral_id,
                    clean_target_id,
                    clean_peripheral_id.contains(&clean_target_id)
                );
                clean_peripheral_id.contains(&clean_target_id)
            } else if is_linux_hci {
                println!("[COMPARE] Linux HCI: {} == {}", peripheral_id, device_id);
                peripheral_id == device_id
            } else {
                println!("[COMPARE] Default: {} == {}", peripheral_id, device_id);
                peripheral_id == device_id
            };

            if is_match {
                println!("[MATCH] Found matching device!");

                // ── Determine config from device name ──────────────────────
                let dev_name = peripheral_props
                    .as_ref()
                    .and_then(|p| p.local_name.clone())
                    .unwrap_or_default();

                apply_ble_device_config(&dev_name);
                // ──────────────────────────────────────────────────────────

                println!("[STATE] Setting BLE_CONNECTED = true");
                *BLE_CONNECTED.lock().unwrap() = true;

                println!("[LSL] Creating outlet ({} channels)...", *BLE_CHANNELS.lock().unwrap());
                if let Err(e) = create_ble_outlet() {
                    println!("[ERROR] Outlet creation failed: {}", e);
                    return Err(format!("LSL initialization failed: {}", e));
                }

                println!("[CONNECT] Attempting connection...");
                let connect_result =
                    tokio::time::timeout(Duration::from_secs(10), peripheral.connect()).await;

                match connect_result {
                    Ok(Ok(_)) => println!("[CONNECT] Connected successfully!"),
                    Ok(Err(e)) => {
                        println!("[ERROR] Connection failed: {}", e);
                        return Err(format!("Connection failed: {}", e));
                    }
                    Err(_) => {
                        println!("[ERROR] Connection timed out");
                        return Err("Connection timed out (10s)".to_string());
                    }
                }

                println!("[SERVICES] Discovering services...");
                if let Err(e) = peripheral.discover_services().await {
                    println!("[ERROR] Service discovery failed: {}", e);
                    return Err(format!("Service discovery failed: {}", e));
                }

                let characteristics = peripheral.characteristics();
                println!("[CHAR] Found {} characteristics", characteristics.len());

                let data_char = characteristics
                    .iter()
                    .find(|c| c.uuid.to_string() == "beb5483e-36e1-4688-b7f5-ea07361b26a8")
                    .ok_or_else(|| {
                        println!("[ERROR] Data characteristic not found");
                        "Data characteristic missing".to_string()
                    })?;

                let control_char = characteristics
                    .iter()
                    .find(|c| c.uuid.to_string() == "0000ff01-0000-1000-8000-00805f9b34fb")
                    .ok_or_else(|| {
                        println!("[ERROR] Control characteristic not found");
                        "Control characteristic missing".to_string()
                    })?;

                println!("[SUBSCRIBE] Setting up notifications...");
                if let Err(e) = peripheral.subscribe(data_char).await {
                    println!("[ERROR] Subscription failed: {}", e);
                    return Err(format!("Notification setup failed: {}", e));
                }

                println!("[CONTROL] Sending start command...");
                if let Err(e) = peripheral
                    .write(control_char, b"start", WriteType::WithResponse)
                    .await
                {
                    println!("[ERROR] Start command failed: {}", e);
                    return Err(format!("Failed to start device: {}", e));
                }

                let mut notifications = match peripheral.notifications().await {
                    Ok(n) => { println!("[NOTIFICATIONS] Stream established"); n }
                    Err(e) => {
                        println!("[ERROR] Notification stream failed: {}", e);
                        return Err(format!("Notification stream error: {}", e));
                    }
                };

                let app_handle_clone = app_handle.clone();

                // ── Spawn data-processing task ─────────────────────────────
                tokio::spawn(async move {
                    println!("[TASK] Starting data processing loop");

                    const EXPECTED_SAMPLE_RATE: f64 = 500.0;
                    const BUFFER_SIZE: usize = 20;

                    let mut sample_count: usize = 0;
                    let mut lost_samples: usize = 0;
                    let mut last_sample_number: Option<u8> = None;
                    let mut last_print_time = Instant::now();
                    let mut buf: VecDeque<f64> = VecDeque::with_capacity(BUFFER_SIZE);

                    while *BLE_CONNECTED.lock().unwrap() {
                        if let Some(data) = notifications.next().await {
                            // Read dynamic lengths inside the loop so they reflect
                            // whatever apply_ble_device_config() set.
                            let single_len = *BLE_SAMPLE_LEN.lock().unwrap();
                            let block_count = 10;
                            let new_packet_len = single_len * block_count;

                            let process_chunk = |chunk: &[u8]| -> Option<(Vec<f32>, u8)> {
                                let current_num = chunk[0];
                                match process_ble_sample(chunk, app_handle_clone.clone()) {
                                    Ok(sample) => Some((sample, current_num)),
                                    Err(e) => { println!("[SAMPLE] {}", e); None }
                                }
                            };

                            let chunks: Vec<&[u8]> = if data.value.len() == new_packet_len {
                                data.value.chunks_exact(single_len).collect()
                            } else if data.value.len() == single_len {
                                vec![&data.value]
                            } else {
                                println!("[WARN] Unexpected packet length: {}", data.value.len());
                                vec![]
                            };

                            for chunk in chunks {
                                if let Some((sample, current_num)) = process_chunk(chunk) {
                                    // Track lost samples
                                    if let Some(last) = last_sample_number {
                                        let expected = last.wrapping_add(1);
                                        if current_num != expected {
                                            let gap = current_num.wrapping_sub(expected) as usize;
                                            lost_samples += gap;
                                            println!("[LOSS] Lost {} sample(s)", gap);
                                        }
                                    }
                                    last_sample_number = Some(current_num);
                                    sample_count += 1;

                                    if let Some(outlet) = &BLE_OUTLET.lock().unwrap().0 {
                                        if let Err(e) = outlet.push_sample(&sample) {
                                            println!("[LSL] Push error: {}", e);
                                        }
                                    }
                                }
                            }

                            // Emit stats once per second
                            let elapsed = last_print_time.elapsed().as_secs_f64();
                            if elapsed >= 1.0 {
                                let actual_rate = sample_count as f64 / elapsed;
                                let expected_samples =
                                    (EXPECTED_SAMPLE_RATE * elapsed) as usize;
                                let received_pct = (sample_count as f64
                                    / expected_samples as f64)
                                    * 100.0;

                                if buf.len() == BUFFER_SIZE { buf.pop_front(); }
                                buf.push_back(received_pct);

                                let _ = app_handle_clone.emit("samplerate", actual_rate);
                                let _ = app_handle_clone.emit("samplelost", lost_samples);
                                let _ = app_handle_clone.emit("connection", "Connected");
                                let _ = app_handle_clone.emit("lsl", "uidbluetooth007");

                                sample_count = 0;
                                lost_samples = 0;
                                last_print_time = Instant::now();
                            }
                        } else {
                            println!("[TASK] Notification stream ended");
                            break;
                        }
                    }

                    println!("[TASK] Cleaning up...");
                    close_ble_outlet();
                });

                return Ok("Connected".to_string());
            }
        }
    }

    println!("[ERROR] Device not found after scanning all adapters");
    Err("Device not found".to_string())
}

// ─── BLE cleanup ─────────────────────────────────────────────────────────────

fn cleanup_resources() {
    println!("[CLEANUP] Performing final cleanup");
    *BLE_CONNECTED.lock().unwrap() = false;
    close_ble_outlet();
}

#[tauri::command]
fn cleanup_ble() {
    close_ble_outlet();
}

fn cleanup_on_exit() {
    println!("[CLEANUP] Application exiting - cleaning up BLE resources");
    if let Some(peripheral) = CONNECTED_PERIPHERAL.lock().unwrap().take() {
        println!("[CLEANUP] Disconnecting peripheral...");
        if let Err(e) = futures::executor::block_on(peripheral.disconnect()) {
            println!("[WARN] Failed to disconnect peripheral: {}", e);
        }
    }
    cleanup_resources();
}

// ─── Entry point ──────────────────────────────────────────────────────────────

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_arduino,
            scan_ble_devices,
            connect_to_ble,
            start_streaming,
            start_wifistreaming,
            cleanup_ble,
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Destroyed = event {
                    cleanup_on_exit();
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
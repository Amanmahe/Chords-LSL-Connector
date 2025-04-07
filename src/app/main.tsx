"use client";

import React, { useRef, useState, useEffect } from 'react';
import { core } from "@tauri-apps/api";
import { Link, Wifi, Bluetooth, X, Minus, ChevronsUp } from 'lucide-react';
import { listen } from "@tauri-apps/api/event";
import { Window } from "@tauri-apps/api/window";

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const portRef = useRef<unknown>(null);
  const [activeButton, setActiveButton] = useState<"serial" | "wifi" | "bluetooth" | null>(null);
  const [devices, setDevices] = useState<{ name: string; id: string }[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);
  const [status, setStatus] = useState("");
  const isProcessing = useRef(false);
  const [alwaysOnTop, setAlwaysOnTop] = useState(false);
  const appWindow = Window.getCurrent();

  const toggleAlwaysOnTop = async () => {
    const newValue = !alwaysOnTop;
    setAlwaysOnTop(newValue);
    await appWindow.setAlwaysOnTop(newValue);
  };

  const minimizeWindow = async () => {
    await appWindow.minimize();
  };

  const closeWindow = async () => {
    await appWindow.close();
  };

  const ConnectserialDevice = async () => {
    try {
      isProcessing.current = true;
      setActiveButton("serial");
      const portName = await core.invoke('detect_arduino') as string;
      portRef.current = portName;
      setDeviceConnected(true);
      await core.invoke('start_streaming', { portName: portRef.current, stream_name: "UDL" });
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  const ConnectwifiDevice = async () => {
    try {
      isProcessing.current = true;
      setActiveButton("wifi");
      setDeviceConnected(true);
      await core.invoke("start_wifistreaming");
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  const ConnectbluetoothDevice = async () => {
    try {
      isProcessing.current = true;
      setActiveButton("bluetooth");
      await core.invoke("scan_ble_devices");
    } catch (error) {
      console.error("Failed to connect to device:", error);
    }
  };

  useEffect(() => {
    listen('bleDevices', (event) => {
      setDevices(event.payload as { name: string; id: string }[]);
    });
  }, []);

  const connectToDevice = async () => {
    if (!selectedDevice) return;
    const response = await core.invoke<string>("connect_to_ble", { deviceId: selectedDevice });
    setStatus(response);
    setDeviceConnected(true);
  };

  const disconnectFromDevice = async () => {
    if (!selectedDevice) return;
    try {
      const response = await core.invoke<string>("disconnect_from_ble", { deviceId: selectedDevice });
      await core.invoke('cleanup_ble');
      setDeviceConnected(false);
      setSelectedDevice(null);
      setActiveButton(null);
      setStatus(response);
    } catch (error) {
      console.error("Failed to disconnect:", error);
      setStatus("Failed to disconnect.");
    }
  };

  return (
    <div className="h-screen flex flex-col bg-gray-200 rounded-2xl overflow-hidden">
      {/* Top Bar */}
      <div className="w-full">
        <div
          className="flex justify-between items-center w-full h-12 px-4 bg-gray-800 text-white select-none"
          data-tauri-drag-region
        >
          {/* Left Buttons */}
          <div className="flex space-x-3">
            <button onClick={activeButton === null ? ConnectserialDevice : undefined} className="hover:text-blue-400" title="Serial">
              <Link size={20} />
            </button>
            {activeButton !== "bluetooth" && (
              <button onClick={activeButton === null ? ConnectbluetoothDevice : undefined} className="hover:text-blue-400" title="Bluetooth">
                <Bluetooth size={20} />
              </button>
            )}
            <button onClick={activeButton === null ? ConnectwifiDevice : undefined} className="hover:text-blue-400" title="WiFi">
              <Wifi size={20} />
            </button>
          </div>

          {/* Right Buttons */}
          <div className="flex space-x-3">
            <button onClick={toggleAlwaysOnTop} className={`${alwaysOnTop ? "text-green-400" : "text-white"} hover:text-green-300`} title="Toggle Always on Top">
              <ChevronsUp size={20} />
            </button>
            <button onClick={minimizeWindow} className="hover:text-yellow-400" title="Minimize">
              <Minus size={20} />
            </button>
            <button onClick={closeWindow} className="hover:text-red-400" title="Close">
              <X size={20} />
            </button>
          </div>
        </div>
      </div>

      {/* Main Content Area */}
      <div className="flex flex-1 w-full px-10 py-6">
        {/* Left Side - Icon and Bluetooth Devices */}
        <div className="flex-1 flex flex-col items-center">
          <div className="w-full h-full flex flex-col items-center justify-center"> {/* Added justify-center here */}
            {/* Icon Container */}
            {!(activeButton === "bluetooth" && !deviceConnected) && (
              <div className="flex items-center justify-center w-40 h-40 rounded-full bg-gray-200 shadow-[8px_8px_16px_#bebebe,-8px_-8px_16px_#ffffff] animate-[rotateShadow_1.5s_linear_infinite]">
                {activeButton ? (
                  {
                    serial: <Link size={50}   className={`transition-colors duration-300 ${deviceConnected && activeButton === "serial" ? "text-green-500" : "text-gray-500"
                    }`}/>,
                    bluetooth: <Bluetooth size={50}   className={`transition-colors duration-300 ${deviceConnected && activeButton === "bluetooth" ? "text-green-500" : "text-gray-500"
                    }`} />,
                    wifi: <Wifi size={50}   className={`transition-colors duration-300 ${deviceConnected && activeButton === "wifi" ? "text-green-500" : "text-gray-500"
                    }`}/>
                  }[activeButton]
                ) : (
                  <div className="text-gray-400 text-sm text-center">
                    <p>Select a</p>
                    <p>connection</p>
                  </div>
                )}
              </div>
            )}

            {/* Disconnect Button (shown only when connected) - Moved outside the inner div */}
            {deviceConnected && (
              <div className="w-full flex justify-center mt-6"> {/* Changed to mt-6 instead of mt-auto */}
                <button
                  onClick={disconnectFromDevice}
                  className="px-6 py-2 bg-red-500 text-black rounded-2xl shadow-lg hover:bg-red-600 transition"
                >
                  Disconnect
                </button>
              </div>
            )}

            {/* Bluetooth Devices List (shown only when scanning) */}
            {activeButton === "bluetooth" && !deviceConnected && (
              <div className="w-[40vh] max-w-md  bg-white rounded-lg shadow-lg p-4 overflow-y-auto max-h-80">
                <h3 className="text-lg font-semibold mb-3 text-center">Available Devices</h3>
                {devices.length > 0 ? (
                  <ul className="space-y-2">
                    {devices.map((device) => (
                      <li key={device.id} className="flex items-center p-2 hover:bg-gray-100 rounded">
                        <input
                          type="radio"
                          id={`device-${device.id}`}
                          name="bluetooth-device"
                          value={device.id}
                          checked={selectedDevice === device.id}
                          onChange={() => setSelectedDevice(device.id)}
                          className="mr-3"
                        />
                        <label htmlFor={`device-${device.id}`} className="flex-1 text-gray-700 cursor-pointer">
                          {device.name || `Unknown Device (${device.id})`}
                        </label>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p className="text-gray-500 py-4 text-center">Scanning for devices...</p>
                )}
                <div className="flex justify-center space-x-4 mt-6">
                  <button
                    onClick={connectToDevice}
                    disabled={!selectedDevice}
                    className={`px-2 py-2 rounded-md ${!selectedDevice
                      ? 'bg-gray-300 cursor-not-allowed'
                      : 'bg-blue-500 hover:bg-blue-600 text-white'
                      } transition-colors`}
                  >
                    Connect
                  </button>
                  <button
                    onClick={() => setActiveButton(null)}
                    className="px-2 py-2 rounded-md bg-gray-200 hover:bg-gray-300 text-gray-700 transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            )}
          </div>


        </div>

        {/* Right Side - Info Cards */}
        <div className="flex-1 flex flex-col justify-center space-y-4 ml-8">
          <div className="bg-white rounded shadow p-2 ">
            <h3 className="text-lg text-black font-semibold">Status</h3>
            <p className="text-gray-700">{status || "No status yet"}</p>
          </div>

          <div className="bg-white rounded shadow p-2">
            <h3 className="text-lg text-black font-semibold">Sample Rate</h3>
            <p className="text-gray-700">500</p>
          </div>

          <div className="bg-white rounded shadow p-2">
            <h3 className="text-lg text-black font-semibold">Sample Lost</h3>
            <p className="text-gray-700">0</p>
          </div>

          <div className="bg-white rounded shadow p-2">
            <h3 className="text-lg text-black font-semibold">LSL</h3>
            <p className="text-gray-700">NPG-Lite</p>
          </div>
        </div>  </div>
    </div>
  );
};

export default App;
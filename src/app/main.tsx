"use client";

import React, { useRef, useState, useEffect } from 'react';
import { core } from "@tauri-apps/api";
import { Link, Wifi, Bluetooth } from 'lucide-react';
import { listen } from "@tauri-apps/api/event"; // ✅ Import listen correctly

const App = () => {
  const [deviceConnected, setDeviceConnected] = useState(false);
  const portRef = useRef<unknown>(null);
  const [activeButton, setActiveButton] = useState<"serial" | "wifi" | "bluetooth" | null>(null);
  const [devices, setDevices] = useState<{ name: string; id: string }[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);
  const [status, setStatus] = useState("");
  const isProcessing = useRef(false);

  const ConnectserialDevice = async () => {
    try {
      isProcessing.current = true;
      setActiveButton("serial");
      const portName = await core.invoke('detect_arduino') as string;
      console.log(`Connected to device on port: ${portName}`);
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
      await core.invoke("scan_bluetooth_devices");


    } catch (error) {
      console.error("Failed to connect to device:", error);
    }
  };

  useEffect(() => {
    listen('bluetoothDevices', (event) => {
      setDevices(event.payload as { name: string; id: string }[]);

      console.log(event.payload); // Check if the devices are printed here
    });
  }, []);

  const connectToDevice = async () => {
    if (!selectedDevice) return;
    const response = await core.invoke<string>("connect_to_bluetooth", { deviceId: selectedDevice });
    setStatus(response);
    setDeviceConnected(true);
  };

  const disconnectFromDevice = async () => {
    if (!selectedDevice) return;
    try {
      const response = await core.invoke<string>("disconnect_from_bluetooth", { deviceId: selectedDevice });
      core.invoke('cleanup_bluetooth')
  .then(() => console.log('Bluetooth cleaned up'))
  .catch(err => console.error('Cleanup failed:', err))
      console.log(response);
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
    <div className="flex flex-col items-center justify-center h-screen bg-gray-200">
      <div className="flex space-x-10 relative">
        {/* Serial Button */}
        <div
          onClick={activeButton === null ? ConnectserialDevice : undefined}
          onMouseDown={activeButton === null ? () => (isProcessing.current = true) : undefined}
          className={`
        flex items-center justify-center w-28 h-28 rounded-full cursor-pointer bg-gray-200 shadow-[8px_8px_16px_#bebebe,-8px_-8px_16px_#ffffff] 
        transition-all duration-600 relative ${activeButton && activeButton !== "wifi" && activeButton !== "bluetooth" ?
              'animate-[rotateShadow_1.5s_linear_infinite]' : ''}
        ${activeButton && activeButton !== "serial" ? "opacity-50 pointer-events-none" : ""}
      `}
        >
          <Link
            size={40}
            className={`transition-colors duration-300 ${deviceConnected && activeButton === "serial" ? "text-green-500" : "text-gray-500"
              }`}
          />
        </div>

        {/* WiFi Button */}
        <div
          onClick={activeButton === null ? ConnectwifiDevice : undefined}
          onMouseDown={activeButton === null ? () => (isProcessing.current = true) : undefined}
          className={`
        flex items-center justify-center w-28 h-28 rounded-full cursor-pointer bg-gray-200 shadow-[8px_8px_16px_#bebebe,-8px_-8px_16px_#ffffff] 
        transition-all duration-600 relative ${activeButton && activeButton !== "serial" && activeButton !== "bluetooth" ?
              'animate-[rotateShadow_1.5s_linear_infinite]' : ''}
        ${activeButton && activeButton !== "wifi" ? "opacity-50 pointer-events-none" : ""}
      `}
        >
          <Wifi
            size={40}
            className={`transition-colors duration-300 ${deviceConnected && activeButton === "wifi" ? "text-green-500" : "text-gray-500"
              }`}
          />
        </div>

        {/* Bluetooth Button */}
        <div
          onClick={activeButton === null ? ConnectbluetoothDevice : undefined}
          onMouseDown={activeButton === null ? () => (isProcessing.current = true) : undefined}
          className={`
        flex items-center justify-center w-28 h-28 rounded-full cursor-pointer bg-gray-200 shadow-[8px_8px_16px_#bebebe,-8px_-8px_16px_#ffffff] 
        transition-all duration-600 relative ${activeButton && activeButton !== "serial" && activeButton !== "wifi" ?
              'animate-[rotateShadow_1.5s_linear_infinite]' : ''}
        ${activeButton && activeButton !== "bluetooth" ? "opacity-50 pointer-events-none" : ""}
      `}
        >
          <Bluetooth
            size={40}
            className={`transition-colors duration-300 ${deviceConnected && activeButton === "bluetooth" ? "text-green-500" : "text-gray-500"
              }`}
          />
        </div>
      </div>

      {/* Bluetooth Devices Popover - Centered on screen */}
      {activeButton === "bluetooth"  && !deviceConnected  && (
        <div className="fixed inset-0 flex items-center justify-center z-50 bg-black bg-opacity-50">
          <div className="bg-white rounded-lg shadow-xl p-6 w-80 max-w-full max-h-[80vh] overflow-y-auto">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-semibold text-gray-800">Bluetooth Devices</h2>
            </div>

            {devices.length > 0 ? (
              <ul className="space-y-2 mb-4">
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

            <div className="flex space-x-3">
              <button
                onClick={connectToDevice}
                disabled={!selectedDevice}
                className={`flex-1 py-2 rounded-md ${!selectedDevice
                  ? 'bg-gray-300 cursor-not-allowed'
                  : 'bg-blue-500 hover:bg-blue-600 text-white'
                  } transition-colors`}
              >
                Connect
              </button>
              <button
                onClick={() => setActiveButton(null)}
                className="flex-1 py-2 rounded-md bg-gray-200 hover:bg-gray-300 text-gray-700 transition-colors"
              >
                Cancel
              </button>
            </div>

            {status && (
              <p className={`mt-3 text-sm ${status.includes("Failed") ? "text-red-500" : "text-gray-600"
                }`}>
                {status}
              </p>
            )}
          </div>

        </div>
      )}
      {deviceConnected && (
        <button
          onClick={disconnectFromDevice}
          className="mt-4 px-6 py-2 bg-red-500 text-black rounded-lg shadow-lg hover:bg-red-600 transition"
        >
          Disconnect
        </button>
      )}
    </div>
  );
};

export default App;

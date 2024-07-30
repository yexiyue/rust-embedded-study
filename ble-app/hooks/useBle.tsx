import { useMemoizedFn } from "ahooks";
import { useMemo, useState } from "react";
import { PermissionsAndroid, Platform } from "react-native";
import { BleManager, Device, ScanMode } from "react-native-ble-plx";
import { useBleStore } from "./useBleStore";

export const manager = new BleManager();

async function requestPermissions() {
  if (Platform.OS === "ios") {
    return true;
  }

  if (Platform.OS === "android") {
    let checks = [
      PermissionsAndroid.check("android.permission.BLUETOOTH_CONNECT"),
      PermissionsAndroid.check("android.permission.BLUETOOTH_SCAN"),
      PermissionsAndroid.check("android.permission.ACCESS_FINE_LOCATION"),
    ];

    let allowed = await Promise.all(checks).then((res) => {
      return res.every((r) => r);
    });

    if (!allowed) {
      const result = await PermissionsAndroid.requestMultiple([
        PermissionsAndroid.PERMISSIONS.BLUETOOTH_SCAN,
        PermissionsAndroid.PERMISSIONS.BLUETOOTH_CONNECT,
        PermissionsAndroid.PERMISSIONS.ACCESS_FINE_LOCATION,
      ]);

      return (
        result["android.permission.BLUETOOTH_CONNECT"] ===
          PermissionsAndroid.RESULTS.GRANTED &&
        result["android.permission.BLUETOOTH_SCAN"] ===
          PermissionsAndroid.RESULTS.GRANTED &&
        result["android.permission.ACCESS_FINE_LOCATION"] ===
          PermissionsAndroid.RESULTS.GRANTED
      );
    }

    return allowed;
  }

  return false;
}

const connectDevice = async (deviceId: Device["id"]) => {
  const device = await manager.connectToDevice(deviceId);
  const connectedDevice = await device.connect();
  return await connectedDevice.discoverAllServicesAndCharacteristics();
};

export const useBle = () => {
  const [device, addDevice] = useBleStore((state) => [
    state.device,
    state.addDevice,
  ]);
  const [isScanning, setIsScanning] = useState(false);

  const startScanDevices = useMemoizedFn(() => {
    if (isScanning) {
      return;
    }
    manager.startDeviceScan(
      null,
      {
        legacyScan: true,
        scanMode: ScanMode.LowPower,
      },
      async (error, device) => {
        if (error) {
          return;
        }
        if (device) {
          setIsScanning(true);
          addDevice(device);
        }
      }
    );
  });

  const devices = useMemo(() => {
    return Object.values(device);
  }, [device]);

  const stopScanDevice = useMemoizedFn(() => {
    setIsScanning(false);
    manager.stopDeviceScan();
  });

  return {
    devices,
    requestPermissions,
    startScanDevices,
    stopScanDevice,
    connectDevice,
  };
};

export function uint8ArrayToBase64(arr: number[]) {
  return btoa(String.fromCharCode.apply(null, arr));
}

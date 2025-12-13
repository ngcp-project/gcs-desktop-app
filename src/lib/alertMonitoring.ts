import { emit } from "@tauri-apps/api/event";
import type { VehicleTelemetryData } from "./bindings";

// ============================================
// Configuration
// ============================================

// Track active alerts with their last update timestamp
const activeAlerts = new Map<string, number>();

// Configurable thresholds
export const ALERT_THRESHOLDS = {
  SIGNAL_STRENGTH: -70,
  LOW_BATTERY: 20,
  PROXIMITY: 100, // feet
};

// Minimum time between alert updates (in milliseconds) preventing excessive toast updates for same alert
const ALERT_UPDATE_DEBOUNCE = 3000; // 3 seconds, toggle for different 

// ============================================
// Helper Functions
// ============================================

/**
 * Generate unique alert key for deduplication
 * This key is also used as the toast ID
 */
const getAlertKey = (vehicle: string, type: string): string => {
  return `${vehicle}_${type}`;
};

/**
 * Calculate distance between two coordinates using Haversine formula
 * @returns Distance in feet
 */
const calculateDistance = (
  lat1: number,
  lon1: number,
  lat2: number,
  lon2: number
): number => {
  const R = 3959; // Earth's radius in miles
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLon = ((lon2 - lon1) * Math.PI) / 180;

  const a =
    Math.sin(dLat / 2) * Math.sin(dLat / 2) +
    Math.cos((lat1 * Math.PI) / 180) *
      Math.cos((lat2 * Math.PI) / 180) *
      Math.sin(dLon / 2) *
      Math.sin(dLon / 2);

  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  return R * c * 5280; // Convert to feet
};

// Emit or update an alert toast 
const emitAlert = (
  vehicle: string,
  type: string,
  severity: "error" | "warning",
  title: string,
  description: string
): void => {
  const alertKey = getAlertKey(vehicle, type);
  const now = Date.now();
  const lastUpdate = activeAlerts.get(alertKey);

  // Check if we should update the alert
  const shouldUpdate = !lastUpdate || (now - lastUpdate) >= ALERT_UPDATE_DEBOUNCE;

  if (shouldUpdate) {
    // Update the timestamp
    activeAlerts.set(alertKey, now);

    // Emit the toast (will update if it already exists due to same ID)
    emit("create-toast", {
      id: alertKey,
      type: severity,
      title,
      description,
    });

    if (!lastUpdate) {
      console.log(`Alert emitted: ${alertKey}`);
    } else {
      console.log(`Alert updated: ${alertKey}`);
    }
  }
  // If not enough time has passed, silently skip (prevents spam)
};

// Clear an alert and dismiss its toast
const clearAlert = (vehicle: string, type: string): void => {
  const alertKey = getAlertKey(vehicle, type);

  if (activeAlerts.has(alertKey)) {
    activeAlerts.delete(alertKey);

    // Dismiss the toast using the same alert key as ID
    emit("dismiss-toast", { id: alertKey });

    console.log(`Alert cleared: ${alertKey}`);
  }
};

// ============================================
// Alert Condition Checkers
// ============================================

// Check signal strength for a vehicle
const checkSignalStrength = (
  vehicle: string,
  signalStrength: number
): void => {
  if (signalStrength < ALERT_THRESHOLDS.SIGNAL_STRENGTH) {
    emitAlert(
      vehicle,
      "signal_strength",
      "warning",
      "Warning: Signal Integrity",
      `Weak signal integrity/connection lost to ${vehicle}!`
    );
  } else {
    clearAlert(vehicle, "signal_strength");
  }
};

// Check connection status for a vehicle 
const checkConnectionStatus = (
  vehicle: string,
  vehicleStatus: string
): void => {
  if (vehicleStatus === "Disconnected") {
    emitAlert(
      vehicle,
      "heartbeat_timeout",
      "error",
      "Error: Connection Failure",
      `Unable to connect to ${vehicle}`
    );
  } else if (vehicleStatus === "Connected") {
    clearAlert(vehicle, "heartbeat_timeout");
  }

  // Also check for "Bad Connection" status
  if (vehicleStatus === "Bad Connection") {
    emitAlert(
      vehicle,
      "signal_strength",
      "warning",
      "Warning: Signal Integrity",
      `Weak signal integrity/connection lost to ${vehicle}!`
    );
  }
};

// Check battery level for a vehicle
const checkBatteryLevel = (vehicle: string, batteryLife: number): void => {
  if (batteryLife < ALERT_THRESHOLDS.LOW_BATTERY) {
    emitAlert(
      vehicle,
      "abnormal_status",
      "error",
      "Error: Abnormal Status",
      `Abnormal ${vehicle} status (low battery: ${batteryLife.toFixed(1)}%)!`
    );
  } else {
    clearAlert(vehicle, "abnormal_status");
  }
};

// Check geo-fence status for a vehicle
const checkGeoFenceStatus = (
  vehicle: string,
  vehicleStatus: string
): void => {
  if (vehicleStatus === "Approaching restricted area") {
    emitAlert(
      vehicle,
      "geo_fence",
      "warning",
      "Warning: Keep-Out Zone",
      `${vehicle} approaching keep-out zone!`
    );
  } else if (vehicleStatus !== "Approaching restricted area") {
    clearAlert(vehicle, "geo_fence");
  }
};

// Check proximity between two vehicles
const checkVehicleProximity = (
  vehicle1: string,
  lat1: number,
  lon1: number,
  vehicle2: string,
  lat2: number,
  lon2: number
): void => {
  const distance = calculateDistance(lat1, lon1, lat2, lon2);

  if (distance < ALERT_THRESHOLDS.PROXIMITY) {
    emitAlert(
      vehicle1,
      `proximity_${vehicle2}`,
      "warning",
      "Warning: Vehicle Proximity",
      `${vehicle1} and ${vehicle2} are within ${distance.toFixed(0)} ft of each other!`
    );
  } else {
    clearAlert(vehicle1, `proximity_${vehicle2}`);
  }
};

// ============================================
// Main Alert Monitoring Function
// ============================================

/**
 * Check all alert conditions for the current telemetry state
 * This is the main entry point called from StoresSync
 */
export const checkAlerts = (
  telemetryState: VehicleTelemetryData | null
): void => {
  if (!telemetryState) return;

  const vehicles = ["ERU", "MEA", "MRA"] as const;

  // Check individual vehicle conditions
  vehicles.forEach((vehicle) => {
    const data = telemetryState[vehicle];
    if (!data) return;

    checkSignalStrength(vehicle, data.signal_strength);
    checkConnectionStatus(vehicle, data.vehicle_status);
    checkBatteryLevel(vehicle, data.battery_life);
    checkGeoFenceStatus(vehicle, data.vehicle_status);
  });

  // Check proximity between vehicles
  if (
    telemetryState.ERU?.current_position &&
    telemetryState.MEA?.current_position
  ) {
    checkVehicleProximity(
      "ERU",
      telemetryState.ERU.current_position.latitude,
      telemetryState.ERU.current_position.longitude,
      "MEA",
      telemetryState.MEA.current_position.latitude,
      telemetryState.MEA.current_position.longitude
    );
  }

  if (
    telemetryState.ERU?.current_position &&
    telemetryState.MRA?.current_position
  ) {
    checkVehicleProximity(
      "ERU",
      telemetryState.ERU.current_position.latitude,
      telemetryState.ERU.current_position.longitude,
      "MRA",
      telemetryState.MRA.current_position.latitude,
      telemetryState.MRA.current_position.longitude
    );
  }

  if (
    telemetryState.MEA?.current_position &&
    telemetryState.MRA?.current_position
  ) {
    checkVehicleProximity(
      "MEA",
      telemetryState.MEA.current_position.latitude,
      telemetryState.MEA.current_position.longitude,
      "MRA",
      telemetryState.MRA.current_position.latitude,
      telemetryState.MRA.current_position.longitude
    );
  }
};

// Get the current set of active alerts
export const getActiveAlerts = (): string[] => {
  return Array.from(activeAlerts.keys());
};

// Clear all active alerts
export const clearAllAlerts = (): void => {
  activeAlerts.forEach((timestamp, alertKey) => {
    emit("dismiss-toast", { id: alertKey });
  });
  activeAlerts.clear();
  console.log("All alerts cleared");
};
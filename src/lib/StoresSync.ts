import { createTauRPCProxy, MissionsStruct } from "./bindings";
import { missionPiniaStore } from "./MissionStore";
import { mapPiniaStore } from "./MapStore";
import { telemetryPiniaStore } from "./TelemetryStore";
import { VehicleTelemetryData } from "./bindings";
import { checkAlerts } from "./alertMonitoring"; 
import { listen } from "@tauri-apps/api/event";

//Declare store variables:
let missionStore: ReturnType<typeof missionPiniaStore>;
let mapStore: ReturnType<typeof mapPiniaStore>;
let telemetryStore: ReturnType<typeof telemetryPiniaStore>;

//Establish taurpc connections.
export const establishTaurpcConnection = () => {
  //Assign stores to their respective useStores():
  missionStore = missionPiniaStore();
  mapStore = mapPiniaStore();
  telemetryStore = telemetryPiniaStore();

  // ===============================================
  // Backend Event Listeners
  // ===============================================
  const taurpc = createTauRPCProxy();
  taurpc.mission.get_all_missions().then((data) => {
    console.log("PINIA: Mission data fetched:", data);
    missionStore!.syncRustState(data);
  });

  taurpc.mission.on_updated.on((data: MissionsStruct) => {
    console.log("PINIA: Mission data updated:", data);
    missionStore!.syncRustState(data);
  });

  taurpc.telemetry.get_telemetry().then((data) => {
    if (!data) {
      taurpc.telemetry.get_default_data().then((defaultData) => {
        telemetryStore.syncRustState(defaultData);
        console.log("PINIA: Telemetry data is empty, using default data", defaultData);
      });
    }
    console.log("PINIA: Telemetry data received", data);
    telemetryStore.syncRustState(data);
  });

  taurpc.telemetry.on_updated.on((data: VehicleTelemetryData) => {
    console.log("PINIA: Telemetry data updated", data);
    telemetryStore.syncRustState(data);
  });
  
  //survivor status update listener, status updates emitted from process.rs
  //updates mission state in-memory for now (integrate with db in the future)
  listen("survivor_status_update", (event) => {
    const { mission_id, vehicle, status, coordinate } = event.payload as {
      mission_id: number;
      vehicle: string;
      status: string;
      coordinate: { lat: number; long: number };
    };
    console.log("PINIA: Survivor status update received:", event.payload);
    
    //get current mission state
    const currentState = missionStore.missionState;
    if (!currentState) {
      console.warn("Survivor update received but no mission state available");
      return;
    }
  
    //find mission to update
    const mission = currentState.missions.find((m) => m.mission_id === mission_id);
    if (!mission) {
      console.warn(`Survivor update received for unknown mission_id: ${mission_id}`);
      return;
    }
  
    //update survivor coordinate on the mission
    mission.survivor_coordinate = {
      lat: coordinate.lat,
      long: coordinate.long
    };
  
    //update appropriate vehicle's patient_status
    const vehicleKey = vehicle as "MEA" | "ERU" | "MRA";
    if (mission.vehicles[vehicleKey]) {
      //cast status to the enum type (Located | Secured | Unsecured)
      mission.vehicles[vehicleKey].patient_status = status as typeof mission.vehicles[typeof vehicleKey]["patient_status"];
    }

    //syncing the updated state
    missionStore.syncRustState(currentState);
  });

  // =============================================
  // Subscriptions
  // =============================================
//Per every update of missionStore, update mapStore.
  missionStore.$subscribe((mutation, state) => {
    mapStore.updateLayerTracking(state.missionState as MissionsStruct);
  });

  //Per every update of telemetryStore, check for alert conditions
  telemetryStore.$subscribe((mutation, state) => {
    checkAlerts(state.telemetryState);
  });
};
// ===============================================
// Movement Simulation --- FOR TESTING ONLY
// Uncomment when testing movement simulation and Uncomment Lines 52-73 and 90 from TelemetryStore.ts
// ===============================================
// let movementInterval: NodeJS.Timeout | undefined;

// export const startMovementSimulation = () => {
//   // Clear any existing interval
//   if (movementInterval) {
//     clearInterval(movementInterval);
//   }

//   movementInterval = setInterval(() => {
//     telemetryStore.simulateMovement();
//   }, 5000); // Update every 5 seconds
// };

// export const stopMovementSimulation = () => {
//   if (movementInterval) {
//     clearInterval(movementInterval);
//     movementInterval = undefined;
//   }
// };

// startMovementSimulation();
export { missionStore, mapStore, telemetryStore };


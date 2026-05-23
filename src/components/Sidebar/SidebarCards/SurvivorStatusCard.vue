<script setup lang="ts">
import { computed } from "vue";
import { Card, CardContent, CardTitle, CardHeader } from "@/components/ui/card";
import { missionStore } from "@/lib/StoresSync";
import type { VehicleEnum } from "@/lib/bindings";

const missionId = computed(() => missionStore.getCurrentMissionId().value);

const currentMission = computed(() => {
  if (missionId.value === null) return null;
  //unwrap ComputedRef with .value
  return missionStore.getMissionData(missionId.value).value;
});

//vehicles that can have survivor status
const engaging_vehicles: VehicleEnum[] = ["MEA", "ERU"];

//NOTE: rust files use "patient_status" field, will be displayed as "Survivor" in UI
//rust files must be updated to "survivor_status" with "Located" enum value later, for now, only "secured" and "unsecured" available

//get all vehicles that have engaged with the survivor
const engagedVehicleNames = computed(() => {
  if (!currentMission.value) return [];
  
  return engaging_vehicles.filter((vehicleName) => {
    const vehicle = currentMission.value?.vehicles[vehicleName];
    // Now checks for both Located and Secured
    return vehicle?.patient_status === "Located" || 
           vehicle?.patient_status === "Secured";
  });
});

const overallStatus = computed(() => {
  if (!currentMission.value) return "Unsecured";
  
  const hasSecured = engaging_vehicles.some(
    (v) => currentMission.value?.vehicles[v]?.patient_status === "Secured"
  );
  
  if (hasSecured) return "Secured";
  
  //check for Located status
  const hasLocated = engaging_vehicles.some(
    (v) => currentMission.value?.vehicles[v]?.patient_status === "Located"
  );
  
  if (hasLocated) return "Located";
  
  return "Unsecured";
});

const survivorCoordinates = computed(() => {
  return currentMission.value?.survivor_coordinate || null;
});
//format engaged vehicles for display
const engagedVehiclesText = computed(() => {
  if (engagedVehicleNames.value.length === 0) return "None";
  return engagedVehicleNames.value.join(", ");
});

const statusStyles = {
  Located: "text-yellow-500 font-semibold",  //for when "Located" is added
  Secured: "text-chart-2 font-semibold",
  Unsecured: "text-destructive font-semibold",
};
</script>

<template>
  <!-- Always visible during active mission -->
  <Card
    v-if="missionId !== null && currentMission"
    class="relative m-2 bg-sidebar-foreground p-2 text-foreground"
  >
    <CardHeader class="pb-2">
      <CardTitle class="text-xl font-bold">Survivor Status</CardTitle>
    </CardHeader>

    <CardContent class="flex flex-col items-start gap-2">
      <!-- Status -->
      <div class="flex items-center gap-2">
        <span class="font-semibold">Status:</span>
        <span :class="statusStyles[overallStatus as keyof typeof statusStyles]">
          {{ overallStatus }}
        </span>
      </div>

      <!-- Engaged Vehicle(s) -->
      <div class="flex items-center gap-2">
        <span class="font-semibold">Engaged by:</span>
        <span :class="engagedVehicleNames.length === 0 ? 'text-muted-foreground' : ''">
          {{ engagedVehiclesText }}
        </span>
      </div>

      <!-- Coordinates/placeholder -->
      <div v-if="survivorCoordinates" class="flex flex-col gap-1">
        <span class="font-semibold">Location:</span>
        <div class="ml-2 text-sm">
          <div>Lat: {{ survivorCoordinates.lat.toFixed(6) }}</div>
          <div>Long: {{ survivorCoordinates.long.toFixed(6) }}</div>
        </div>
      </div>
      
      <!-- Placeholder message when no coordinates yet -->
      <div v-else class="text-sm text-muted-foreground italic">
        Awaiting survivor location...
      </div>
    </CardContent>
  </Card>
</template>
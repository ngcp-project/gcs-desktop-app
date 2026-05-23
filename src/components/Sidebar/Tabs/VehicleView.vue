<script setup lang="ts">
import VehicleCard from "@/components/Sidebar/SidebarCards/VehicleCard.vue";
import SurvivorStatusCard from "@/components/Sidebar/SidebarCards/SurvivorStatusCard.vue";
import { SidebarContent, SidebarGroup } from "@/components/ui/sidebar";
import { VehicleEnum } from "@/lib/bindings";
import { missionStore } from "@/lib/StoresSync";

const handleClick = (vehicleName: VehicleEnum) => {
  missionStore.setCurrentView("stage");
  missionStore.setCurrentVehicleName(vehicleName);
};
</script>

<template>
  <SidebarContent class="bg-sidebar-background">
    <SidebarGroup class="vehicles-list">
      <VehicleCard vehicleName="ERU" @click="handleClick('ERU')" />
      <VehicleCard vehicleName="MEA" @click="handleClick('MEA')" />
      <VehicleCard vehicleName="MRA" @click="handleClick('MRA')" />
      <SurvivorStatusCard />
    </SidebarGroup>
  </SidebarContent>
</template>

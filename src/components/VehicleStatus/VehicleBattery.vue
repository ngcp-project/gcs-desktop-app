<script lang="ts" setup>
import { defineProps, computed } from "vue";

const {percentage} = defineProps<{
  percentage: number;
  charging: boolean;
}>();
const batteryColor = computed(() => {
  if (percentage <= 20) return { border: "border-red-500", bg: "bg-red-500" };
  return { border: "border-black", bg: "bg-black" };
});

</script>

<template>
  <div class="relative flex flex-col h-full w-full items-center">
    <!-- Smaller tip -->
    <div :class="`h-[1px] w-1 border ${batteryColor.border}`"></div> <!-- Reduced width/height -->
    
    <!-- Smaller main battery body -->
    <div :class="`relative flex h-4 w-3 rounded-[2px] border-[1.5px] ${batteryColor.border} items-end`"> <!-- Halved dimensions -->
      <!-- Battery fill (dynamic height) -->
      <div
        :class="`${batteryColor.bg} w-full`"
        :style="{ height: Math.max(percentage, 5) + '%' }"
      />
      
      <!-- Lightning icon (scaled down) -->
      <div v-if="charging" class="absolute flex h-full w-full items-center justify-center">
        <img class="h-3/5 w-3/5" src="..\..\assets\lightning-icon-png-5.png" /> <!-- Reduced icon size -->
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Animation styles (unchanged) */
.dead {
  visibility: visible;
  animation: blinker 1s linear infinite;
}
.charging {
  visibility: visible;
}

@keyframes blinker {
  50% {
    opacity: 0;
  }
}
</style>
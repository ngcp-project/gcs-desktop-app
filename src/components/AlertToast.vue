<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { emit } from "@tauri-apps/api/event";
import { useRoute } from "vue-router";
import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { Toaster } from "@/components/ui/sonner";
import { toast } from "vue-sonner";

// --------- Toast Position Based on Route --------- //
const route = useRoute();
const toasterPosition = computed(() => {
  return route.path === "/" ? "bottom-right" : "bottom-left"; // '/' is the route of StaticScreen
});

// Position Dismiss All button based on route
const dismissButtonPosition = computed(() => {
  return route.path === "/" 
    ? "fixed bottom-4 right-4 z-50"     // Camera Screen
    : "fixed bottom-4 left-4 z-50";     // Overview Screen
});

// --------- Listen for Alert Events --------- //
// These events are emitted by alertMonitoring.ts when alert conditions are detected

listen("create-toast", (event) => {
  const { id, type, title, description } = event.payload as {
    id: string;
    type: "error" | "warning" | "info";
    title: string;
    description: string;
  };

  toast[type](title, {
    id,
    description,
    duration: Infinity,
    action: { label: "Dismiss", onClick: () => emit("dismiss-toast", { id }) }
  });
});

listen("dismiss-toast", (event) => {
  const { id } = event.payload as { id: string };
  toast.dismiss(id);
});

listen("dismiss-all-toasts", () => {
  toast.dismiss();
});
</script>

<template>
  <Toaster 
    richColors 
    :position="toasterPosition"
  />

  <!-- ========================================= -->
  <!-- Dismiss All Button - Positioned by Route -->
  <!-- ========================================= -->
  <Button
    :class="dismissButtonPosition"
    variant="outline"
    @click="
      () => {
        emit('dismiss-all-toasts');
      }
    "
  >
    Dismiss All
  </Button>

  <!-- ========================================= -->
  <!-- Testing Buttons - Remove in Demo   -->
  <!-- ========================================= -->
  
  <!-- --------- Errors --------- -->
   <!--
  <Button
    variant="outline"
    @click="
      () => {
        emit('create-toast', {
          id: 'test_connection_error',
          type: 'error',
          title: 'Error: Connection Failure',
          description: 'Unable to connect to ERU (test)'
        });
      }
    "
  >
    Test Connection Error
  </Button>

  <Button
    variant="outline"
    @click="
      () => {
        emit('create-toast', {
          id: 'test_abnormal_status',
          type: 'error',
          title: 'Error: Abnormal Status',
          description: 'Abnormal MEA status (low battery) (test)'
        });
      }
    "
  >
    Test Abnormal Status Error
    
  </Button>
  -->

  <!-- --------- Warnings --------- -->
   <!--
  <Button
    variant="outline"
    @click="
      () => {
        emit('create-toast', {
          id: 'test_signal_integrity',
          type: 'warning',
          title: 'Warning: Signal Integrity',
          description: 'Weak signal integrity/connection lost to MRA (test)'
        });
      }
    "
  >
    Test Signal Integrity Warning
  </Button>

  <Button
    variant="outline"
    @click="
      () => {
        emit('create-toast', {
          id: 'test_keep_out',
          type: 'warning',
          title: 'Warning: Keep-Out Zone',
          description: 'ERU within 500 ft of keep-out zone (test)'
        });
      }
    "
  >
    Test Keep-Out Warning
  </Button>

  <Button
    variant="outline"
    @click="
      () => {
        emit('create-toast', {
          id: 'test_proximity',
          type: 'warning',
          title: 'Warning: Vehicle Proximity',
          description: 'ERU and MEA are within 50 ft of each other (test)'
        });
      }
    "
  >
    Test Proximity Warning
  </Button>
-->
</template>


<!-- Toast offset and reduction in size -->
<style scoped>
/* Offset toasts from bottom only, without affecting horizontal position */
:deep([data-sonner-toaster]) {
  bottom: 60px !important;
}

/* Make toasts smaller */
:deep([data-sonner-toast]) {
  max-width: 320px !important;      /* Reduce width (default is 356px) */
  padding: 12px !important;          /* Reduce padding (default is 16px) */
  font-size: 0.875rem !important;   /* Smaller font size (14px) */
}

/* Adjust toast title size */
:deep([data-sonner-toast] [data-title]) {
  font-size: 0.875rem !important;   /* Smaller title (14px) */
  margin-bottom: 2px !important;     /* Less space below title */
}

/* Adjust toast description size */
:deep([data-sonner-toast] [data-description]) {
  font-size: 0.8125rem !important;  /* Smaller description (13px) */
  line-height: 1.3 !important;       /* Tighter line height */
}

/*  Adjust toast button size */
:deep([data-sonner-toast] button) {
  font-size: 0.8125rem !important;  /* Smaller button text (13px) */
  padding: 4px 8px !important;       /* Smaller button padding */
}
</style>
<script setup lang="ts">
import { emit } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
</script>

<template>
  <div class="fixed bottom-0 z-50 flex w-full items-center justify-center gap-3 pb-3">
    <!-- Errors -->
    <Button
      variant="outline"
      @click="
        emit('create-toast', {
          id: 'test_connection_error',
          type: 'error',
          title: 'Error: Connection Failure',
          description: 'Unable to connect to ERU (test)',
        })
      "
    >
      Connection Error
    </Button>

    <Button
      variant="outline"
      @click="
        emit('create-toast', {
          id: 'test_abnormal_status',
          type: 'error',
          title: 'Error: Abnormal Status',
          description: 'Abnormal MEA status (low battery) (test)',
        })
      "
    >
      Abnormal Status Error
    </Button>

    <!-- Warnings -->
    <Button
      variant="outline"
      @click="
        emit('create-toast', {
          id: 'test_signal_integrity',
          type: 'warning',
          title: 'Warning: Signal Integrity',
          description: 'Weak signal integrity/connection lost to MRA (test)',
        })
      "
    >
      Signal Integrity Warning
    </Button>

    <Button
      variant="outline"
      @click="
        emit('create-toast', {
          id: 'test_keep_out',
          type: 'warning',
          title: 'Warning: Keep-Out Zone',
          description: 'ERU within 500 ft of keep-out zone (test)',
        })
      "
    >
      Keep-Out Warning
    </Button>

    <Button
      variant="outline"
      @click="
        emit('create-toast', {
          id: 'test_proximity',
          type: 'warning',
          title: 'Warning: Vehicle Proximity',
          description: 'ERU and MEA are within 50 ft of each other (test)',
        })
      "
    >
      Proximity Warning
    </Button>

    <!-- Dismiss All -->
    <Button
      variant="outline"
      @click="emit('dismiss-all-toasts')"
    >
      Dismiss All
    </Button>
  </div>
</template>
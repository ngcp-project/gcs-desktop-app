<script setup lang="ts">
import { ref, computed, reactive } from "vue";
import { listen } from "@tauri-apps/api/event";
import { emit } from "@tauri-apps/api/event";
import { useRoute } from "vue-router";

// Types
interface ToastData {
  id: string;
  type: "error" | "warning" | "info";
  title: string;
  description: string;
}

// Toast Position Based on Route
const route = useRoute();
const isRight = computed(() => route.path === "/" );

// Reactive Toast State
const activeToasts = ref<Record<string, ToastData>>({});
const disabledToasts = reactive<Record<string,number>>({}); 
const isHovered = ref(false);

// Max number of toasts visible when collapsed (the rest are fully hidden)
const VISIBLE_COLLAPSED = 3;

const startCooldown = (id:string): void => {
  disabledToasts[id] = 3;

  const tick = (remaining: number) =>{
    if(remaining <= 0){
      delete disabledToasts[id];
      return;
    }
    disabledToasts[id] = remaining;
    setTimeout(() => tick(remaining-1),1000);
  };

  setTimeout(()=>tick(2),1000);
}

const dismissCooldown = (toast:ToastData):number=>{
  if(toast.type!="error") return 0;
  return disabledToasts[toast.id] ?? 0;
}


// Sorted toast list for rendering: Warnings first, errors last 
const sortedToasts = computed(() => {
  return Object.values(activeToasts.value).sort((a, b) => {
    const getPriority = (toast: any): number => {
      //use explicit priority
      if (toast.priority !== undefined) return toast.priority;

      //fallback
      return toast.type === "error" ? 1 : 0;
    }
    return getPriority(b) - getPriority(a);
  });
});

// Index from the front (0 = bottom/most prominent) */
const getStackIndex = (arrayIndex: number): number => {
  return sortedToasts.value.length - 1 - arrayIndex;
};

const isVisible = (arrayIndex:number): boolean=>{
  return getStackIndex(arrayIndex) < VISIBLE_COLLAPSED;
};

const addOrUpdateToast = (toast: ToastData): void => {
  if(!activeToasts.value[toast.id] && toast.type === "error"){
    startCooldown(toast.id)
  }
  activeToasts.value = { ...activeToasts.value, [toast.id]: toast };
};

const removeToast = (id: string): void => {
  if (activeToasts.value[id]) {
    const { [id]: _, ...rest } = activeToasts.value;
    activeToasts.value = rest;
    delete disabledToasts[id];
  }
};

const clearAll = (): void => {
  const remaining: Record<string, ToastData> = {};

  for (const [id, toast] of Object.entries(activeToasts.value)) {
    if (toast.type === "error" && disabledToasts[id] > 0) {
      remaining[id] = toast;
    } else {
      delete disabledToasts[id];
    }
  }

  activeToasts.value = remaining;
  console.log("All dismissable alerts cleared");
};

const dismissToast = (id: string): void => {
  emit("dismiss-toast", { id });
};

// Tauri Event Listeners

listen("create-toast", (event) => {
  const payload = event.payload as ToastData;
  addOrUpdateToast(payload);
});

listen("dismiss-toast", (event) => {
  const { id } = event.payload as { id: string };
  removeToast(id);
});

listen("dismiss-all-toasts", () => {
  clearAll();
});
</script>

<template>
  <div
    :class="[
      'toast-stack',
      isRight ? 'toast-stack--right' : 'toast-stack--left',
      { 'toast-stack--expanded': isHovered },
    ]"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
  >
    <!-- "stage" is a relative container that collapses to
      the height of a single toast when not hovered. -->
    <div class="toast-stage">
      <TransitionGroup name="toast">
        <div
          v-for="(t, index) in sortedToasts"
          :key="t.id"
          :class="[
            'toast-item',
            `toast-item--${t.type}`,
            {
              'toast-item--hidden': !isHovered && !isVisible(index),
              'toast-item--front': getStackIndex(index) === 0,
            },
          ]"
          :style="{
            '--stack-index': getStackIndex(index),
          }"
          role="alert"
        >
          <!-- Icon -->
          <div class="toast-icon">
            <svg v-if="t.type === 'error'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"/>
              <path d="m15 9-6 6"/>
              <path d="m9 9 6 6"/>
            </svg>
            <svg v-else-if="t.type === 'warning'" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/>
              <path d="M12 9v4"/>
              <path d="M12 17h.01"/>
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 16v-4"/>
              <path d="M12 8h.01"/>
            </svg>
          </div>

          <!-- Content -->
          <div class="toast-content">
            <div class="toast-title">{{ t.title }}</div>
            <div class="toast-description">{{ t.description }}</div>
          </div>

          <!-- Dismiss -->
          <button class="toast-dismiss" 
          :class = "{'toast-dismiss--disabled': dismissCooldown(t) > 0}"
          :disabled="dismissCooldown(t) > 0"
          @click="dismissToast(t.id)">
          Dismiss{{ dismissCooldown(t) > 0 ? ` (${dismissCooldown(t)})` : '' }}
          </button>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<style scoped>
/*Stack container — fixed to viewport*/
.toast-stack {
  position: fixed;
  bottom: 60px;
  z-index: 9999;
  pointer-events: none;
}

.toast-stack--right {
  right: 16px;
}

.toast-stack--left {
  left: 16px;
}

/* Stage — wrapper that toasts position themselves inside of */
.toast-stage {
  position: relative;
  pointer-events: none;
}

/* ---- Front toast: always in flow to size the container ---- */
.toast-item--front {
  position: relative;
  z-index: 1;
}

/* ---- Non-front toasts: always absolute, never switch ---- */
.toast-item:not(.toast-item--front) {
  position: absolute;
  bottom: 0;
  left: 0;
}

/* ---- Collapsed (default): peek behind front toast ---- */
.toast-stack:not(.toast-stack--expanded) .toast-item:not(.toast-item--front) {
  height: 72px;
  overflow: hidden;
  transform:
    translateY(calc(var(--stack-index) * -8px))
    scale(calc(1 - var(--stack-index) * 0.04));
  opacity: calc(1 - var(--stack-index) * 0.15);
}

.toast-stack--expanded .toast-stage {
  display: flex;
  flex-direction: column-reverse;
  gap: 8px;
  padding-top: 16px; 
}

.toast-stack--expanded .toast-item {
  position: relative !important;
  transform: none !important;
  opacity: 1 !important;
  bottom: auto !important;
  left: auto !important;
}

.toast-stack--expanded::before {
  content: '';
  position: absolute;
  bottom: 0;
  left: -20px;
  right: -20px;
  height: 800px;  /* tall enough to cover any reasonable stack */
  pointer-events: auto;
}

.toast-stack--left .toast-stage {
  align-items: flex-start;
}

/* Front toast stays in flow so the container has a size */
.toast-stack:not(.toast-stack--expanded) .toast-item--front {
  position: relative;
  z-index: 1;
}

.toast-item--hidden {
  opacity: 0 !important;
  pointer-events: none !important;
}

/* ---------- Type colors ---------- */
.toast-item--error {
  background-color: #f6cbcb;
  border-color: #f48080;
  color: #991b1b;
}

.toast-item--warning {
  background-color: #fffbeb;
  border-color: #fcd34d;
  color: #92400e;
}

.toast-item--info {
  background-color: #eff6ff;
  border-color: #93c5fd;
  color: #1e40af;
}

/** Individual toast */
.toast-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  max-width: 320px;
  min-width: 280px;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  pointer-events: auto;
  transform-origin: bottom center;
  transition:
    transform 0.35s cubic-bezier(0.16, 1, 0.3, 1),
    opacity 0.3s ease;
}

/* ---------- Inner elements ---------- */
.toast-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.toast-content {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-size: 0.875rem;
  font-weight: 600;
  line-height: 1.3;
  margin-bottom: 2px;
}

.toast-description {
  font-size: 0.8125rem;
  line-height: 1.3;
  opacity: 0.9;
}

.toast-dismiss {
  flex-shrink: 0;
  align-self: center;
  font-size: 0.8125rem;
  font-weight: 500;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid currentColor;
  background: transparent;
  color: inherit;
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 0.15s ease;
   min-width: 5.5rem;   /* wide enough to fit "Dismiss (3)" */
  text-align: center;
}

.toast-dismiss:hover {
  opacity: 1;
}

.toast-dismiss--disabled {
  opacity: 0.3;
  cursor: not-allowed;
  pointer-events: none;
}

/* ---------- Transition animations ---------- */
.toast-enter-active {
  transition: all 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-stack .toast-leave-active {
  opacity: 0;
  visibility: hidden;
  position: absolute !important;
  z-index: -1;
  pointer-events: none;
}

.toast-enter-from {
  opacity: 0;
  transform: translateY(10px) scale(0.95);
}

.toast-leave-to {
  opacity: 0;
}

.toast-move {
  transition: transform 0.3s cubic-bezier(0.25, 0.1, 0.25, 1);
}

</style>
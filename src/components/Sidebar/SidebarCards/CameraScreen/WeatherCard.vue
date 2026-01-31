<script setup lang="ts">
import { Card, CardContent } from "@/components/ui/card";
import overcast from "@/assets/weather-icons/overcast.png";
import wind from "@/assets/weather-icons/wind.png";
import rain from "@/assets/weather-icons/rain.png";
const weatherStyles = "flex items-center gap-1";
</script>

<template>
  <Card class="m-2 h-fit bg-sidebar-foreground p-2 text-foreground">
    <CardContent class="mt-1 flex flex-col items-start space-y-3">
      <div :class=weatherStyles>
        <img :src="overcast" /> placeHolder Overcast
      </div>
      <div :class=weatherStyles>
        <img :src="wind" /> placeHolder Mph
      </div>
      <div :class=weatherStyles>
        <img :src="rain" /> placeHolder in/hr
      </div>
    </CardContent>
  </Card>
</template>

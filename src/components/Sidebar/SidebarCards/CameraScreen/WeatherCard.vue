<script setup lang="ts">
  import { Card, CardContent } from "@/components/ui/card";
  import CloudCoverage from "./WeatherIcons/CloudCoverage.vue";
  import Wind from "./WeatherIcons/Wind.vue";
  import Rain from "./WeatherIcons/Rain.vue";
  import NullData from "./WeatherIcons/NullData.vue";
  
  interface WeatherData {
    overcast: string,
    wind: number,
    rain: number
  }
  
  /* 
    Component | Chill (black) -> warning (Orange) -> unacceptable (Red)
    ---------------------------
    CloudCoverage  | Sunny -> Cloudy
    Wind      | 0 mph -> idk mph
    Rain      | no rain -> heavy rain
  */


 const mockWeatherData: WeatherData = {
   overcast: "sunny",
   wind: 5, //mph
   rain: 5 // in/h
  }; //get from state manager
  const levels = {
    acceptable: "fill-black",
    warning: "fill-yellow-500",
    unacceptable: "fill-red-500"
  };
  const cloudCoverageStates = {
    sun: "sun",
    partial: "partial",
    cloud: "cloud"
  }
  let cloudCoverageState = cloudCoverageStates.sun;
  //compute appropriate color
  let overcastStatus = levels.warning;
  let windStatus  = (mockWeatherData.wind <= 8) ? levels.acceptable : (mockWeatherData.wind <= 10) ? levels.warning : levels.unacceptable;
  let rainStatus = (mockWeatherData.rain <= 0.01) ? levels.acceptable : ( mockWeatherData.rain <= 0.02) ? levels.warning : levels.unacceptable;
  const weatherStyles = "flex items-center gap-1";
</script>
<template>
  <Card class="m-2 h-fit bg-sidebar-foreground p-2 text-foreground">
    <CardContent class="mt-1 flex flex-col items-start space-y-3">
      <div :class=weatherStyles>
          <CloudCoverage :state="cloudCoverageState"/> {{ mockWeatherData.overcast }}
      </div>
      <div :class=weatherStyles>
          <Wind :color="windStatus"/> {{ mockWeatherData.wind }} mph
      </div>
      <div :class=weatherStyles>
        <Rain :color="rainStatus"/> {{ mockWeatherData.rain }}in/h
      </div>
    </CardContent>
  </Card>
</template>

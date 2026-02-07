<script setup lang="ts">
  import { Card, CardContent } from "@/components/ui/card";
  import Overcast from "./WeatherIcons/Overcast.vue";
  import Wind from "./WeatherIcons/Wind.vue";
  import Rain from "./WeatherIcons/Rain.vue";
  import NullData from "./WeatherIcons/NullData.vue";
  
  interface WeatherData {
    overcast: string,
    wind: string,
    rain: string
  }
  
  /* 
  Component | Chill (black) -> moderate (Orange) -> Severe (Red)
  ---------------------------
  Overcast  | Sunny -> Cloudy
  Wind      | 0 mph -> idk mph
  Rain      | no rain -> heavy rain
  
  
  */
 const mockWeatherData: WeatherData = {
   overcast: "sunny",
   wind: "strong",
   rain: "heavy"
  }; //get from state manager
  const levels = {
    light: "black",
    moderate: "orange",
    severe: "red"
  };
  
  //compute appropriate color
  let overcastStatus = mockWeatherData.overcast == ("sunny") ? levels.light : levels.severe;
  let windStatus  = mockWeatherData.wind == ("strong") ? levels.moderate : levels.severe;
  let rainStatus = mockWeatherData.rain == ("heavy") ? levels.severe : levels.severe;
  const weatherStyles = "flex items-center gap-1";
</script>
<template>
  <Card class="m-2 h-fit bg-sidebar-foreground p-2 text-foreground">
    <CardContent class="mt-1 flex flex-col items-start space-y-3">
      <div :class=weatherStyles>
          <Overcast :color="overcastStatus"/> placeholder
      </div>
      <div :class=weatherStyles>
          <Wind :color="windStatus"/> placeholder MPH
      </div>
      <div :class=weatherStyles>
        <Rain :color="rainStatus"/> placeholder
      </div>
    </CardContent>
  </Card>
</template>

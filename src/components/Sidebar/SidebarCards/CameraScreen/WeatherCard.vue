<script setup lang="ts">
  import { Card, CardContent } from "@/components/ui/card";
  import Overcast from "./WeatherIcons/Overcast.vue";
  import Wind from "./WeatherIcons/Wind.vue";
  import Rain from "./WeatherIcons/Rain.vue";
  import NullData from "./WeatherIcons/NullData.vue";
  
  interface WeatherData {
    overcast: string,
    wind: number,
    rain: number
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
   wind: 5,
   rain: 5
  }; //get from state manager
  const levels = {
    light: "black",
    moderate: "orange",
    severe: "red"
  };
  
  //compute appropriate color
  let overcastStatus = mockWeatherData.overcast == ("sunny") ? levels.light : levels.severe;
  let windStatus  = mockWeatherData.wind == (5) ? levels.moderate : levels.severe;
  let rainStatus = mockWeatherData.rain == (5) ? levels.severe : levels.severe;
  const weatherStyles = "flex items-center gap-1";
</script>
<template>
  <Card class="m-2 h-fit bg-sidebar-foreground p-2 text-foreground">
    <CardContent class="mt-1 flex flex-col items-start space-y-3">
      <div :class=weatherStyles>
          <Overcast :color="overcastStatus"/> {{ mockWeatherData.overcast }}
      </div>
      <div :class=weatherStyles>
          <Wind :color="windStatus"/> {{ mockWeatherData.wind }} mph
      </div>
      <div :class=weatherStyles>
        <Rain :color="rainStatus"/> {{ mockWeatherData.rain }}%
      </div>
    </CardContent>
  </Card>
</template>

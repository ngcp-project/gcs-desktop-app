// Original work Copyright (c) 2012 Ardhi Lukianto
// Modified work Copyright (c) 2023 Northrop Grumman Collaboration Project
// Distributed under the MIT License.
// https://github.com/ardhi/Leaflet.MousePosition/blob/c32f1c84ec49dbf7ad599c51c8659d5e08af0f97/src/L.Control.MousePosition.js

<script lang="ts">
import L, { LeafletMouseEvent } from "leaflet";

interface CoordinatesControlOptions extends L.ControlOptions {
  digits?: number;
}

interface CoordinatesControl extends L.Control {
  options: CoordinatesControlOptions;
  _container: HTMLElement;
  _onMouseMove: (e: LeafletMouseEvent) => void;
}

const Coordinates = L.Control.extend({
  options: {
    position: "bottomleft",
    digits: 4,
  } as CoordinatesControlOptions,

  _onMouseMove: function(this: CoordinatesControl, e: LeafletMouseEvent) {
    var lat = L.Util.formatNum(e.latlng.lat, this.options.digits).toFixed(this.options.digits);
    var lng = L.Util.formatNum(e.latlng.lng, this.options.digits).toFixed(this.options.digits);
    var html = "Lat: " + lat + "<br />" + "Lng: " + lng;
    this._container.innerHTML = html;
  },

  onAdd: function(this: CoordinatesControl, map: L.Map): HTMLElement {
    this._container = L.DomUtil.create('div', 'leaflet-control-mouseposition');
    L.DomEvent.disableClickPropagation(this._container);
    map.on('mousemove', this._onMouseMove, this);
    this._container.innerHTML = "";
    return this._container;
  },

  onRemove: function (this: CoordinatesControl, map: L.Map): void {
    map.off('mousemove', this._onMouseMove)
  },
});

export { Coordinates }
</script>

<style lang="css">
.leaflet-container .leaflet-control-mouseposition {
  background-color: #FFFFFFB0;
  padding: 0.05em 0.3em;
  margin: 16px;
  color: #000;
  border-radius: 4px;
  box-shadow: 0 0 4px #000;
  font-size: 1.6em;
}
</style>
// Original work Copyright (c) 2012 Ardhi Lukianto
// Modified work Copyright (c) 2023 Northrop Grumman Collaboration Project
// Distributed under the MIT License.
// https://github.com/ardhi/Leaflet.MousePosition/blob/c32f1c84ec49dbf7ad599c51c8659d5e08af0f97/src/L.Control.MousePosition.js

import L, { LeafletMouseEvent } from "leaflet";
import "./MapCoordinates.css";

interface CoordinatesControlOptions extends L.ControlOptions {
  digits?: number;
}

interface CoordinatesControl extends L.Control {
  options: CoordinatesControlOptions;
  _container: HTMLElement;
  _onMouseMove: (e: LeafletMouseEvent) => void;
}

export const Coordinates = L.Control.extend({
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
    this._container.innerHTML = "Lat: " + "<br />" + "Lng: ";
    return this._container;
  },

  onRemove: function (this: CoordinatesControl, map: L.Map): void {
    map.off('mousemove', this._onMouseMove)
  },
});

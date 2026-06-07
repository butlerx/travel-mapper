// @ts-check
/// <reference path="globals.d.ts" />
import { createMap, arcPoints, typeColors } from './map-core.js';

/** @type {HTMLElement | null} */
const el = document.getElementById('journey-map');
if (el && typeof L !== 'undefined') {
  const oLat = parseFloat(el.dataset.originLat || '');
  const oLng = parseFloat(el.dataset.originLng || '');
  const dLat = parseFloat(el.dataset.destLat || '');
  const dLng = parseFloat(el.dataset.destLng || '');
  if (!isNaN(oLat) && !isNaN(dLat)) {
    const map = createMap('journey-map', { scrollWheelZoom: false });
    const color = typeColors[el.dataset.type || 'air'] || typeColors.air;
    const points = arcPoints([oLat, oLng], [dLat, dLng], 50);
    L.marker([oLat, oLng]).addTo(map);
    L.marker([dLat, dLng]).addTo(map);
    L.polyline(points, { color, weight: 3, opacity: 0.85 }).addTo(map);
    map.fitBounds(points, { padding: [40, 40] });
  }
}

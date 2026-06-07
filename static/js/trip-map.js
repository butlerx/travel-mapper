// @ts-check
/// <reference path="globals.d.ts" />
import { createMap, arcPoints, typeColors } from './map-core.js';

/** @type {HTMLElement | null} */
const el = document.getElementById('trip-map');
if (el && typeof L !== 'undefined') {
  /** @type {Array<{oLat: number, oLng: number, dLat: number, dLng: number, type?: string}>} */
  const legs = JSON.parse(el.dataset.legs || '[]');
  if (legs.length > 0) {
    const map = createMap('trip-map', { scrollWheelZoom: false });

    /** @type {Array<[number, number]>} */
    const allPoints = [];
    /** @type {Set<string>} */
    const markerKeys = new Set();

    for (const leg of legs) {
      const color = typeColors[leg.type || 'air'] || typeColors.air;
      const points = arcPoints([leg.oLat, leg.oLng], [leg.dLat, leg.dLng], 50);
      L.polyline(points, { color, weight: 3, opacity: 0.85 }).addTo(map);

      const oKey = `${leg.oLat},${leg.oLng}`;
      const dKey = `${leg.dLat},${leg.dLng}`;
      if (!markerKeys.has(oKey)) {
        L.marker([leg.oLat, leg.oLng]).addTo(map);
        markerKeys.add(oKey);
      }
      if (!markerKeys.has(dKey)) {
        L.marker([leg.dLat, leg.dLng]).addTo(map);
        markerKeys.add(dKey);
      }

      for (const p of points) allPoints.push(p);
    }

    map.fitBounds(allPoints, { padding: [40, 40] });
  }
}

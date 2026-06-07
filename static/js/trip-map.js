// @ts-check
/// <reference path="globals.d.ts" />
import { createMap, arcPoints, typeColors } from './map-core.js';

/** @type {HTMLElement | null} */
const el = document.getElementById('trip-map');
if (el && typeof maplibregl !== 'undefined') {
  /** @type {Array<{oLat: number, oLng: number, dLat: number, dLng: number, type?: string}>} */
  const legs = JSON.parse(el.dataset.legs || '[]');
  if (legs.length > 0) {
    const map = createMap('trip-map', { scrollZoom: false });

    map.on('load', () => {
      const lineFeatures = legs.map((leg, i) => ({
        type: 'Feature',
        properties: { id: i, color: typeColors[leg.type || 'air'] || typeColors.air },
        geometry: {
          type: 'LineString',
          coordinates: arcPoints([leg.oLng, leg.oLat], [leg.dLng, leg.dLat], 60),
        },
      }));

      /** @type {Set<string>} */
      const seen = new Set();
      const pointFeatures = [];
      for (const leg of legs) {
        const oKey = `${leg.oLng},${leg.oLat}`;
        const dKey = `${leg.dLng},${leg.dLat}`;
        if (!seen.has(oKey)) {
          pointFeatures.push({ type: 'Feature', geometry: { type: 'Point', coordinates: [leg.oLng, leg.oLat] }, properties: {} });
          seen.add(oKey);
        }
        if (!seen.has(dKey)) {
          pointFeatures.push({ type: 'Feature', geometry: { type: 'Point', coordinates: [leg.dLng, leg.dLat] }, properties: {} });
          seen.add(dKey);
        }
      }

      map.addSource('trip-arcs', { type: 'geojson', data: { type: 'FeatureCollection', features: lineFeatures } });
      map.addLayer({
        id: 'trip-arc-lines',
        type: 'line',
        source: 'trip-arcs',
        paint: { 'line-color': ['get', 'color'], 'line-width': 2.5, 'line-opacity': 0.85 },
        layout: { 'line-cap': 'round', 'line-join': 'round' },
      });

      map.addSource('trip-points', { type: 'geojson', data: { type: 'FeatureCollection', features: pointFeatures } });
      map.addLayer({
        id: 'trip-endpoint-circles',
        type: 'circle',
        source: 'trip-points',
        paint: { 'circle-radius': 5, 'circle-color': '#56b4e9', 'circle-stroke-width': 1.5, 'circle-stroke-color': '#fff' },
      });

      const bounds = new maplibregl.LngLatBounds();
      for (const leg of legs) {
        bounds.extend([leg.oLng, leg.oLat]);
        bounds.extend([leg.dLng, leg.dLat]);
      }
      map.fitBounds(bounds, { padding: 60 });
    });
  }
}

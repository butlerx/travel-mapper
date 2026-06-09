import { createMap, arcPoints, typeColors } from './map-core';

declare const maplibregl: any;

const el = document.getElementById('journey-map');
if (el && typeof maplibregl !== 'undefined') {
  const oLat = parseFloat(el.dataset.originLat || '');
  const oLng = parseFloat(el.dataset.originLng || '');
  const dLat = parseFloat(el.dataset.destLat || '');
  const dLng = parseFloat(el.dataset.destLng || '');
  if (!isNaN(oLat) && !isNaN(dLat)) {
    const map = createMap('journey-map', { scrollZoom: false, center: [oLng, oLat], zoom: 3 });
    const color = typeColors[el.dataset.type || 'air'] || typeColors.air;
    const points = arcPoints([oLng, oLat], [dLng, dLat], 60);

    map.on('load', () => {
      map.addSource('arc', {
        type: 'geojson',
        data: {
          type: 'Feature',
          geometry: { type: 'LineString', coordinates: points },
          properties: {},
        },
      });
      map.addLayer({
        id: 'arc-line',
        type: 'line',
        source: 'arc',
        paint: { 'line-color': color, 'line-width': 2.5, 'line-opacity': 0.85 },
        layout: { 'line-cap': 'round', 'line-join': 'round' },
      });
      map.addSource('endpoints', {
        type: 'geojson',
        data: {
          type: 'FeatureCollection',
          features: [
            {
              type: 'Feature',
              geometry: { type: 'Point', coordinates: [oLng, oLat] },
              properties: {},
            },
            {
              type: 'Feature',
              geometry: { type: 'Point', coordinates: [dLng, dLat] },
              properties: {},
            },
          ],
        },
      });
      map.addLayer({
        id: 'endpoint-circles',
        type: 'circle',
        source: 'endpoints',
        paint: {
          'circle-radius': 5,
          'circle-color': color,
          'circle-stroke-width': 1.5,
          'circle-stroke-color': '#fff',
        },
      });
      map.fitBounds(new maplibregl.LngLatBounds([oLng, oLat], [dLng, dLat]), { padding: 60 });
    });
  }
}

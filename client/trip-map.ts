import { createMap, arcPoints, typeColors } from './map-core';

declare const maplibregl: any;

const el = document.getElementById('trip-map');
if (el && typeof maplibregl !== 'undefined') {
  const legs: { oLat: number; oLng: number; dLat: number; dLng: number; type?: string }[] =
    JSON.parse(el.dataset.legs || '[]');
  if (legs.length > 0) {
    const map = createMap('trip-map', { scrollZoom: false });

    map.on('load', () => {
      const lineFeatures = legs.map((leg, i) => ({
        type: 'Feature' as const,
        properties: { id: i, color: typeColors[leg.type || 'air'] || typeColors.air },
        geometry: {
          type: 'LineString' as const,
          coordinates: arcPoints([leg.oLng, leg.oLat], [leg.dLng, leg.dLat], 60),
        },
      }));

      const seen = new Set<string>();
      const pointFeatures: any[] = [];
      for (const leg of legs) {
        for (const [lng, lat] of [
          [leg.oLng, leg.oLat],
          [leg.dLng, leg.dLat],
        ]) {
          const k = `${lng},${lat}`;
          if (!seen.has(k)) {
            pointFeatures.push({
              type: 'Feature',
              geometry: { type: 'Point', coordinates: [lng, lat] },
              properties: {},
            });
            seen.add(k);
          }
        }
      }

      map.addSource('trip-arcs', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: lineFeatures },
      });
      map.addLayer({
        id: 'trip-arc-lines',
        type: 'line',
        source: 'trip-arcs',
        paint: { 'line-color': ['get', 'color'], 'line-width': 2.5, 'line-opacity': 0.85 },
        layout: { 'line-cap': 'round', 'line-join': 'round' },
      });
      map.addSource('trip-points', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: pointFeatures },
      });
      map.addLayer({
        id: 'trip-endpoint-circles',
        type: 'circle',
        source: 'trip-points',
        paint: {
          'circle-radius': 5,
          'circle-color': '#56b4e9',
          'circle-stroke-width': 1.5,
          'circle-stroke-color': '#fff',
        },
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

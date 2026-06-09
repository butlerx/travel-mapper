import { createMap, escapeHtml } from './map-core';
import { country_alpha2, visit_color } from '/static/wasm/map_wasm.js';

declare const maplibregl: any;
declare const topojson: any;

const counts: Record<string, number> = JSON.parse(
  document.getElementById('country-counts')?.textContent || '{}',
);
const mapEl = document.getElementById('stats-map');

if (mapEl && Object.keys(counts).length > 0) {
  const map = createMap('stats-map', { center: [10, 30], zoom: 2, scrollZoom: false });

  const tooltipEl = document.createElement('div');
  tooltipEl.className = 'stats-map-tooltip';
  tooltipEl.style.display = 'none';
  mapEl.appendChild(tooltipEl);

  const legendEl = document.createElement('div');
  legendEl.className = 'stats-map-legend';
  const grades = [1, 2, 3, 5, 10, 15, 25, 50];
  const labels = [
    '1',
    '2',
    '3\u20134',
    '5\u20139',
    '10\u201314',
    '15\u201324',
    '25\u201349',
    '50+',
  ];
  legendEl.innerHTML =
    '<strong>Visits</strong>' +
    grades
      .map(
        (g, i) =>
          `<div class="stats-map-legend-row"><span class="stats-map-legend-swatch" style="background:${visit_color(g)}"></span>${labels[i]}</div>`,
      )
      .join('');
  mapEl.appendChild(legendEl);

  map.on('load', async () => {
    try {
      const r = await fetch('https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json');
      const topo = await r.json();
      const geo = topojson.feature(topo, topo.objects.countries);

      geo.features.forEach((f: any) => {
        const id = f.id || f.properties?.id;
        const alpha2 = country_alpha2(String(id));
        f.properties = f.properties || {};
        f.properties.visit_count = counts[alpha2] || 0;
        f.properties.name = f.properties.name || alpha2 || 'Unknown';
      });

      map.addSource('countries', { type: 'geojson', data: geo });
      map.addLayer({
        id: 'countries-fill',
        type: 'fill',
        source: 'countries',
        paint: {
          'fill-color': [
            'step',
            ['get', 'visit_count'],
            'transparent',
            1,
            '#473677',
            2,
            '#482878',
            3,
            '#3e4989',
            5,
            '#31688e',
            10,
            '#26828e',
            15,
            '#1f9e89',
            25,
            '#6ece58',
            50,
            '#fde724',
          ],
          'fill-opacity': ['case', ['>', ['get', 'visit_count'], 0], 0.75, 0],
        },
      });
      map.addLayer({
        id: 'countries-outline',
        type: 'line',
        source: 'countries',
        paint: {
          'line-color': [
            'case',
            ['>', ['get', 'visit_count'], 0],
            'rgba(255,255,255,0.4)',
            'transparent',
          ],
          'line-width': ['case', ['>', ['get', 'visit_count'], 0], 0.8, 0],
        },
      });

      map.on('mousemove', 'countries-fill', (e: any) => {
        if (!e.features?.length) return;
        const { name, visit_count } = e.features[0].properties;
        tooltipEl.innerHTML = `<strong>${escapeHtml(name)}</strong><br>${visit_count} visit${visit_count === 1 ? '' : 's'}`;
        tooltipEl.style.display = 'block';
        map.getCanvas().style.cursor = visit_count > 0 ? 'pointer' : '';
        if (visit_count > 0) {
          map.setPaintProperty('countries-outline', 'line-width', [
            'case',
            ['==', ['id'], e.features[0].id],
            2,
            ['>', ['get', 'visit_count'], 0],
            0.8,
            0,
          ]);
        }
      });
      map.on('mouseleave', 'countries-fill', () => {
        tooltipEl.style.display = 'none';
        map.getCanvas().style.cursor = '';
        map.setPaintProperty('countries-outline', 'line-width', [
          'case',
          ['>', ['get', 'visit_count'], 0],
          0.8,
          0,
        ]);
      });
    } catch {}
  });
}

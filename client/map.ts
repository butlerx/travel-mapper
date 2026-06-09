import { createMap, cssVar } from './map-core';
import {
  compute_routes,
  journey_sidebar_html,
  route_popup_html,
  city_popup_html,
} from '/static/wasm/map_wasm.js';

declare const maplibregl: any;

interface HopResponse {
  id: number;
  travel_type: string;
  origin_name: string;
  origin_lat: number;
  origin_lng: number;
  dest_name: string;
  dest_lat: number;
  dest_lng: number;
  start_date: string;
  [k: string]: any;
}

const initialJourneys: HopResponse[] = JSON.parse(
  document.getElementById('initial-journeys')!.textContent || '[]',
);
const map = createMap('map', { center: [10, 48], zoom: 4 });
const popup = new maplibregl.Popup({ closeButton: true, closeOnClick: true, maxWidth: '360px' });
let routeIndexJson = '{}';

function renderJourneyCards(journeys: HopResponse[]) {
  const sidebar = document.getElementById('journey-sidebar');
  if (sidebar) sidebar.innerHTML = journey_sidebar_html(JSON.stringify(journeys));
}

function renderJourneys(journeys: HopResponse[]) {
  const data = JSON.parse(compute_routes(JSON.stringify(journeys)));
  routeIndexJson = JSON.stringify(data.route_index || {});

  const bounds = new maplibregl.LngLatBounds();
  for (const f of data.arc_features.features) {
    const coords = f.geometry.coordinates;
    if (coords.length > 0) {
      bounds.extend(coords[0]);
      bounds.extend(coords[coords.length - 1]);
    }
  }

  if (map.getSource('routes')) {
    map.getSource('routes').setData(data.arc_features);
    map.getSource('cities').setData(data.city_features);
  } else {
    map.addSource('routes', { type: 'geojson', data: data.arc_features });
    map.addLayer({
      id: 'route-lines',
      type: 'line',
      source: 'routes',
      paint: {
        'line-color': ['get', 'color'],
        'line-width': ['get', 'width'],
        'line-opacity': ['get', 'opacity'],
      },
      layout: { 'line-cap': 'round', 'line-join': 'round' },
    });
    map.addSource('cities', { type: 'geojson', data: data.city_features });
    map.addLayer({
      id: 'city-circles',
      type: 'circle',
      source: 'cities',
      paint: {
        'circle-radius': ['interpolate', ['linear'], ['get', 'count'], 1, 4, 5, 6, 20, 8],
        'circle-color': cssVar('--color-type-air'),
        'circle-opacity': 0.85,
        'circle-stroke-width': 1.5,
        'circle-stroke-color': '#1e3a5f',
      },
    });

    map.on('click', 'route-lines', (e: any) => {
      if (!e.features?.length) return;
      const html = route_popup_html(e.features[0].properties.key, routeIndexJson);
      if (html) popup.setLngLat(e.lngLat).setHTML(html).addTo(map);
    });
    map.on('click', 'city-circles', (e: any) => {
      if (!e.features?.length) return;
      const { name, count, routes } = e.features[0].properties;
      popup
        .setLngLat(e.features[0].geometry.coordinates)
        .setHTML(city_popup_html(name, count, routes))
        .addTo(map);
    });
    map.on('mouseenter', 'route-lines', () => {
      map.getCanvas().style.cursor = 'pointer';
    });
    map.on('mouseleave', 'route-lines', () => {
      map.getCanvas().style.cursor = '';
    });
    map.on('mouseenter', 'city-circles', () => {
      map.getCanvas().style.cursor = 'pointer';
    });
    map.on('mouseleave', 'city-circles', () => {
      map.getCanvas().style.cursor = '';
    });
  }

  document.getElementById('journey-count')!.textContent = `${data.journey_count} journeys`;
  if (!bounds.isEmpty()) map.fitBounds(bounds, { padding: 40 });
}

document.getElementById('toggle-routes')?.addEventListener('change', (e) => {
  try {
    map.setLayoutProperty(
      'route-lines',
      'visibility',
      (e.target as HTMLInputElement).checked ? 'visible' : 'none',
    );
  } catch {}
});
document.getElementById('toggle-airports')?.addEventListener('change', (e) => {
  try {
    map.setLayoutProperty(
      'city-circles',
      'visibility',
      (e.target as HTMLInputElement).checked ? 'visible' : 'none',
    );
  } catch {}
});

map.on('load', () => {
  renderJourneys(initialJourneys);
  renderJourneyCards(initialJourneys);
});

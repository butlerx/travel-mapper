import init, { arc_points, night_polygon, escape_html } from '/static/wasm/map_wasm.js';

await init();

declare const maplibregl: any;
declare const pmtiles: any;
declare const basemaps: any;

const PMTILES_URL = 'pmtiles://https://build.protomaps.com/20260601.pmtiles';
const SATELLITE_URL =
  'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}';
const NIGHT_LIGHTS_URL =
  'https://map1.vis.earthdata.nasa.gov/wmts-webmerc/VIIRS_CityLights_2012/default/GoogleMapsCompatible_Level8/{z}/{y}/{x}.jpg';
const STORAGE_KEY = 'travel-mapper-satellite';
const LIGHTS_KEY = 'travel-mapper-nightlights';

const rootStyles = getComputedStyle(document.documentElement);

export function cssVar(name: string): string {
  return rootStyles.getPropertyValue(name).trim();
}

export const typeColors: Record<string, string> = {
  air: cssVar('--color-type-air'),
  rail: cssVar('--color-type-rail'),
  boat: cssVar('--color-type-boat'),
  transport: cssVar('--color-type-transport'),
};

export { escape_html as escapeHtml };

export function arcPoints(
  from: [number, number],
  to: [number, number],
  numPoints: number,
): [number, number][] {
  return arc_points(from[0], from[1], to[0], to[1], numPoints);
}

function nightPolygon(): object {
  return JSON.parse(night_polygon());
}

export function createMap(containerId: string, options?: Record<string, unknown>) {
  const protocol = new pmtiles.Protocol();
  maplibregl.addProtocol('pmtiles', protocol.tile);

  const styleLayers = basemaps.layers('protomaps', basemaps.namedFlavor('dark'), { lang: 'en' });

  const map = new maplibregl.Map({
    container: containerId,
    style: {
      version: 8,
      glyphs: 'https://protomaps.github.io/basemaps-assets/fonts/{fontstack}/{range}.pbf',
      sprite: 'https://protomaps.github.io/basemaps-assets/sprites/v4/dark',
      sources: {
        protomaps: {
          type: 'vector',
          url: PMTILES_URL,
          attribution:
            '<a href="https://protomaps.com">Protomaps</a> &copy; <a href="https://openstreetmap.org">OpenStreetMap</a>',
        },
        satellite: {
          type: 'raster',
          tiles: [SATELLITE_URL],
          tileSize: 256,
          attribution: '&copy; Esri',
        },
        nightlights: {
          type: 'raster',
          tiles: [NIGHT_LIGHTS_URL],
          tileSize: 256,
          maxzoom: 8,
          attribution: '&copy; NASA VIIRS',
        },
      },
      layers: [
        {
          id: 'satellite-tiles',
          type: 'raster',
          source: 'satellite',
          layout: { visibility: 'none' },
        },
        ...styleLayers,
      ],
    },
    maxZoom: 18,
    renderWorldCopies: true,
    ...(options || {}),
  });

  map.on('style.load', () => map.setProjection({ type: 'mercator' }));
  map.addControl(new maplibregl.NavigationControl({ visualizePitch: true }), 'top-left');
  if (maplibregl.GlobeControl) map.addControl(new maplibregl.GlobeControl(), 'top-left');

  const ctrl = document.createElement('div');
  ctrl.className = 'maplibregl-ctrl maplibregl-ctrl-group';
  const satBtn = Object.assign(document.createElement('button'), {
    type: 'button',
    className: 'satellite-toggle-btn',
    title: 'Toggle satellite imagery',
    textContent: '\uD83D\uDEF0',
  });
  const lightsBtn = Object.assign(document.createElement('button'), {
    type: 'button',
    className: 'satellite-toggle-btn',
    title: 'Toggle city lights',
    textContent: '\uD83C\uDF03',
  });

  let satActive = false;
  let lightsActive = true;
  let opaqueIds: string[] = [];

  const applySat = () => {
    satBtn.classList.toggle('is-active', satActive);
    try {
      localStorage.setItem(STORAGE_KEY, satActive ? '1' : '0');
    } catch {}
    try {
      map.setLayoutProperty('satellite-tiles', 'visibility', satActive ? 'visible' : 'none');
      for (const id of opaqueIds)
        map.setLayoutProperty(id, 'visibility', satActive ? 'none' : 'visible');
    } catch {}
  };
  const applyLights = () => {
    lightsBtn.classList.toggle('is-active', lightsActive);
    try {
      localStorage.setItem(LIGHTS_KEY, lightsActive ? '1' : '0');
    } catch {}
    try {
      map.setLayoutProperty('night-lights', 'visibility', lightsActive ? 'visible' : 'none');
    } catch {}
  };

  satBtn.addEventListener('click', () => {
    satActive = !satActive;
    applySat();
  });
  lightsBtn.addEventListener('click', () => {
    lightsActive = !lightsActive;
    applyLights();
  });
  ctrl.append(satBtn, lightsBtn);
  map.addControl({ onAdd: () => ctrl, onRemove: () => {} }, 'top-left');

  map.on('load', () => {
    const layers: any[] = map.getStyle().layers;
    opaqueIds = layers
      .filter(
        (l: any) => (l.type === 'fill' || l.type === 'background') && l.id !== 'satellite-tiles',
      )
      .map((l: any) => l.id);
    const symbolIds = layers.filter((l: any) => l.type === 'symbol').map((l: any) => l.id);
    for (const id of symbolIds) {
      map.setPaintProperty(id, 'text-color', '#ffffff');
      map.setPaintProperty(id, 'text-halo-color', 'rgba(0,0,0,0.7)');
      map.setPaintProperty(id, 'text-halo-width', 1.5);
    }

    map.addSource('night', { type: 'geojson', data: nightPolygon() });
    map.addLayer({
      id: 'night-overlay',
      type: 'fill',
      source: 'night',
      paint: { 'fill-color': '#000', 'fill-opacity': 0.35 },
    });
    map.addLayer({
      id: 'night-lights',
      type: 'raster',
      source: 'nightlights',
      paint: { 'raster-opacity': 0.6 },
    });

    let lastUpdate = 0;
    const tick = (ts: number) => {
      if (ts - lastUpdate > 10000) {
        lastUpdate = ts;
        try {
          map.getSource('night').setData(nightPolygon());
        } catch {}
      }
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);

    try {
      satActive = localStorage.getItem(STORAGE_KEY) === '1';
    } catch {}
    if (satActive) applySat();
    try {
      lightsActive = localStorage.getItem(LIGHTS_KEY) !== '0';
    } catch {}
    applyLights();
  });

  return map;
}

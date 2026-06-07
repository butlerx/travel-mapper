// @ts-check
/// <reference path="globals.d.ts" />

/**
 * Shared map core — MapLibre GL JS v5 + Protomaps dark basemap + deck.gl.
 *
 * Centralises basemap creation, globe projection, and shared utilities so a
 * single change lands on every map surface at once.
 */

/** Protomaps daily PMTiles build. Update the date to pick up new map data. */
const PMTILES_URL = 'pmtiles://https://build.protomaps.com/20260601.pmtiles';

const SATELLITE_URL =
  'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}';

const NIGHT_LIGHTS_URL =
  'https://map1.vis.earthdata.nasa.gov/wmts-webmerc/VIIRS_CityLights_2012/default/GoogleMapsCompatible_Level8/{z}/{y}/{x}.jpg';

const rootStyles = getComputedStyle(document.documentElement);

/**
 * @param {string} name
 * @returns {string}
 */
export function cssVar(name) {
  return rootStyles.getPropertyValue(name).trim();
}

/** Travel-type → accent colour, resolved from the shared design tokens. */
export const typeColors = {
  air: cssVar('--color-type-air'),
  rail: cssVar('--color-type-rail'),
  boat: cssVar('--color-type-boat'),
  transport: cssVar('--color-type-transport'),
};

/**
 * @param {string} str
 * @returns {string}
 */
export function escapeHtml(str) {
  const div = document.createElement('div');
  div.appendChild(document.createTextNode(str));
  return div.innerHTML;
}

/**
 * Great-circle distance in kilometres.
 * @param {number} lat1
 * @param {number} lng1
 * @param {number} lat2
 * @param {number} lng2
 * @returns {number}
 */
export function haversineKm(lat1, lng1, lat2, lng2) {
  const R = 6371;
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLng = ((lng2 - lng1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) * Math.sin(dLat / 2) +
    Math.cos((lat1 * Math.PI) / 180) *
      Math.cos((lat2 * Math.PI) / 180) *
      Math.sin(dLng / 2) *
      Math.sin(dLng / 2);
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
}

/**
 * Interpolate points along the great-circle path between two [lng, lat] pairs,
 * unwrapping longitude so the line does not jump across the antimeridian.
 * @param {[number, number]} from - [lng, lat]
 * @param {[number, number]} to   - [lng, lat]
 * @param {number} numPoints
 * @returns {[number, number][]} array of [lng, lat]
 */
export function arcPoints(from, to, numPoints) {
  const lat1 = (from[1] * Math.PI) / 180;
  const lng1 = (from[0] * Math.PI) / 180;
  const lat2 = (to[1] * Math.PI) / 180;
  let lng2 = (to[0] * Math.PI) / 180;

  const dLng = lng2 - lng1;
  if (dLng > Math.PI) lng2 -= 2 * Math.PI;
  else if (dLng < -Math.PI) lng2 += 2 * Math.PI;

  const d =
    2 *
    Math.asin(
      Math.sqrt(
        Math.pow(Math.sin((lat1 - lat2) / 2), 2) +
          Math.cos(lat1) * Math.cos(lat2) * Math.pow(Math.sin((lng1 - lng2) / 2), 2),
      ),
    );

  if (d < 1e-10) return [from, to];

  const points = [];
  let prevLng = from[0];
  for (let i = 0; i <= numPoints; i++) {
    const f = i / numPoints;
    const A = Math.sin((1 - f) * d) / Math.sin(d);
    const B = Math.sin(f * d) / Math.sin(d);
    const x = A * Math.cos(lat1) * Math.cos(lng1) + B * Math.cos(lat2) * Math.cos(lng2);
    const y = A * Math.cos(lat1) * Math.sin(lng1) + B * Math.cos(lat2) * Math.sin(lng2);
    const z = A * Math.sin(lat1) + B * Math.sin(lat2);
    const lat = (Math.atan2(z, Math.sqrt(x * x + y * y)) * 180) / Math.PI;
    let lng = (Math.atan2(y, x) * 180) / Math.PI;

    while (lng - prevLng > 180) lng -= 360;
    while (lng - prevLng < -180) lng += 360;
    prevLng = lng;

    points.push(/** @type {[number, number]} */ ([lng, lat]));
  }
  return points;
}

/**
 * Register the PMTiles protocol and create a MapLibre map with the Protomaps
 * dark basemap, globe projection, and navigation controls.
 * @param {string} containerId
 * @param {Record<string, unknown>} [options]
 * @returns {maplibregl.Map}
 */
export function createMap(containerId, options) {
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
          attribution: '&copy; Esri &mdash; Maxar, Earthstar Geographics',
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
        { id: 'satellite-tiles', type: 'raster', source: 'satellite', layout: { visibility: 'none' } },
        ...styleLayers,
      ],
    },
    maxZoom: 18,
    renderWorldCopies: true,
    ...(options || {}),
  });

  map.on('style.load', () => {
    map.setProjection({ type: 'mercator' });
  });

  map.addControl(new maplibregl.NavigationControl({ visualizePitch: true }), 'top-left');

  if (maplibregl.GlobeControl) {
    map.addControl(new maplibregl.GlobeControl(), 'top-left');
  }

  map.addControl(new SatelliteToggle(), 'top-left');

  return map;
}

const STORAGE_KEY = 'travel-mapper-satellite';
const LIGHTS_KEY = 'travel-mapper-nightlights';

/**
 * Compute a GeoJSON polygon covering the night half of the Earth.
 * Uses the solar declination and sub-solar longitude for the current UTC time.
 * @returns {GeoJSON.Feature}
 */
function nightPolygon() {
  const now = new Date();
  const start = new Date(now.getFullYear(), 0, 1);
  const dayOfYear = Math.floor((now.getTime() - start.getTime()) / 86400000);
  const declRad = (-23.44 * Math.PI) / 180 * Math.cos((2 * Math.PI * (dayOfYear + 10)) / 365);
  const utcH = now.getUTCHours() + now.getUTCMinutes() / 60 + now.getUTCSeconds() / 3600;
  const solarLng = -((utcH - 12) * 15);

  const ring = [];
  for (let lng = -180; lng <= 180; lng += 0.5) {
    const ha = ((lng - solarLng) * Math.PI) / 180;
    const lat = (Math.atan(-Math.cos(ha) / Math.tan(declRad)) * 180) / Math.PI;
    ring.push([lng, lat]);
  }
  const nightPole = declRad >= 0 ? -90 : 90;
  ring.push([180, nightPole], [-180, nightPole]);

  return { type: 'Feature', geometry: { type: 'Polygon', coordinates: [ring] }, properties: {} };
}

class SatelliteToggle {
  /** @type {boolean} */
  _active = false;
  /** @type {boolean} */
  _lights = true;
  /** @type {string[]} */
  _opaqueLayerIds = [];
  /** @type {string[]} */
  _symbolLayerIds = [];

  onAdd(/** @type {maplibregl.Map} */ map) {
    const container = document.createElement('div');
    container.className = 'maplibregl-ctrl maplibregl-ctrl-group';

    const satBtn = document.createElement('button');
    satBtn.type = 'button';
    satBtn.className = 'satellite-toggle-btn';
    satBtn.title = 'Toggle satellite imagery';
    satBtn.textContent = '\uD83D\uDEF0';

    const lightsBtn = document.createElement('button');
    lightsBtn.type = 'button';
    lightsBtn.className = 'satellite-toggle-btn';
    lightsBtn.title = 'Toggle city lights';
    lightsBtn.textContent = '\uD83C\uDF03';

    const applySat = () => {
      satBtn.classList.toggle('is-active', this._active);
      try { localStorage.setItem(STORAGE_KEY, this._active ? '1' : '0'); } catch {}
      try {
        map.setLayoutProperty('satellite-tiles', 'visibility', this._active ? 'visible' : 'none');
        for (const id of this._opaqueLayerIds) {
          map.setLayoutProperty(id, 'visibility', this._active ? 'none' : 'visible');
        }
      } catch {}
    };

    const applyLights = () => {
      lightsBtn.classList.toggle('is-active', this._lights);
      try { localStorage.setItem(LIGHTS_KEY, this._lights ? '1' : '0'); } catch {}
      try {
        map.setLayoutProperty('night-lights', 'visibility', this._lights ? 'visible' : 'none');
      } catch {}
    };

    satBtn.addEventListener('click', () => { this._active = !this._active; applySat(); });
    lightsBtn.addEventListener('click', () => { this._lights = !this._lights; applyLights(); });

    map.on('load', () => {
      const layers = map.getStyle().layers;
      this._opaqueLayerIds = layers
        .filter((l) => (l.type === 'fill' || l.type === 'background') && l.id !== 'satellite-tiles')
        .map((l) => l.id);
      this._symbolLayerIds = layers
        .filter((l) => l.type === 'symbol')
        .map((l) => l.id);

      for (const id of this._symbolLayerIds) {
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
      const updateNight = (/** @type {number} */ ts) => {
        if (ts - lastUpdate > 10000) {
          lastUpdate = ts;
          try { /** @type {any} */ (map.getSource('night')).setData(nightPolygon()); } catch {}
        }
        requestAnimationFrame(updateNight);
      };
      requestAnimationFrame(updateNight);

      try { this._active = localStorage.getItem(STORAGE_KEY) === '1'; } catch {}
      if (this._active) applySat();

      try { this._lights = localStorage.getItem(LIGHTS_KEY) !== '0'; } catch {}
      applyLights();
    });

    container.appendChild(satBtn);
    container.appendChild(lightsBtn);
    return container;
  }

  onRemove() {}
}

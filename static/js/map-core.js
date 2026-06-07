// @ts-check
/// <reference path="globals.d.ts" />

/**
 * Shared map core for every Leaflet surface in the app (dashboard, stats,
 * journey detail, trip detail). Centralises the dark basemap, map defaults,
 * and great-circle arc maths so a single styling change lands on all maps at
 * once and they cannot drift apart again.
 */

/** Dark CARTO raster basemap used across every map surface. */
export const TILE_URL = 'https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}@2x.png';

/** @type {Record<string, unknown>} */
export const TILE_OPTIONS = {
  attribution:
    '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
  maxZoom: 19,
  subdomains: 'abcd',
};

/** Leaflet map options shared by every surface. */
export const MAP_DEFAULTS = {
  worldCopyJump: true,
  maxBounds: [
    [-85, -Infinity],
    [85, Infinity],
  ],
  maxBoundsViscosity: 1.0,
  minZoom: 2,
};

/**
 * Create a Leaflet map on the given element with shared defaults and the dark
 * basemap already attached.
 * @param {string} id
 * @param {Record<string, unknown>} [options]
 * @returns {L.Map}
 */
export function createMap(id, options) {
  const map = L.map(id, { ...MAP_DEFAULTS, ...(options || {}) });
  L.tileLayer(TILE_URL, TILE_OPTIONS).addTo(map);
  return map;
}

/**
 * @param {string} str
 * @returns {string}
 */
export function escapeHtml(str) {
  const div = document.createElement('div');
  div.appendChild(document.createTextNode(str));
  return div.innerHTML;
}

const rootStyles = getComputedStyle(document.documentElement);

/**
 * Read a CSS custom property off the document root.
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
 * Interpolate points along the great-circle path between two coordinates,
 * unwrapping longitude so the line does not jump across the antimeridian.
 * @param {[number, number]} from
 * @param {[number, number]} to
 * @param {number} numPoints
 * @returns {[number, number][]}
 */
export function arcPoints(from, to, numPoints) {
  const lat1 = (from[0] * Math.PI) / 180;
  const lng1 = (from[1] * Math.PI) / 180;
  const lat2 = (to[0] * Math.PI) / 180;
  let lng2 = (to[1] * Math.PI) / 180;

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
  let prevLng = from[1];
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

    points.push(/** @type {[number, number]} */ ([lat, lng]));
  }
  return points;
}

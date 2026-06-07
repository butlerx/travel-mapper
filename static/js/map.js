// @ts-check
/// <reference path="types.d.ts" />
import { createMap, escapeHtml, haversineKm, arcPoints, cssVar, typeColors } from './map-core.js';

/** @type {HopResponse[]} */
const initialJourneys = JSON.parse(document.getElementById('initial-journeys').textContent || '[]');

const map = createMap('map', { center: [10, 48], zoom: 4 });

/** @type {Record<string, string>} */
const emojis = {
  air: '\u2708\uFE0F',
  rail: '\uD83D\uDE86',
  boat: '\uD83D\uDEA2',
  transport: '\uD83D\uDE97',
};

/** @type {Record<string, string>} */
const fallbackIcons = {
  air: '/static/icons/plane.svg',
  rail: '/static/icons/train.svg',
  boat: '/static/icons/boat.svg',
  transport: '/static/icons/transport.svg',
};

/** @type {Record<string, string>} */
const carrierDomains = {
  'aer lingus': 'aerlingus.com',
  aeroflot: 'aeroflot.ru',
  'air france': 'airfrance.com',
  'air canada': 'aircanada.com',
  'alaska airlines': 'alaskaair.com',
  'american airlines': 'aa.com',
  'asiana airlines': 'flyasiana.com',
  asiana: 'flyasiana.com',
  'british airways': 'britishairways.com',
  'cathay pacific': 'cathaypacific.com',
  delta: 'delta.com',
  'delta air lines': 'delta.com',
  easyjet: 'easyjet.com',
  emirates: 'emirates.com',
  etihad: 'etihad.com',
  'etihad airways': 'etihad.com',
  finnair: 'finnair.com',
  iberia: 'iberia.com',
  'ita airways': 'ita-airways.com',
  ita: 'ita-airways.com',
  'japan airlines': 'jal.com',
  jal: 'jal.com',
  jetblue: 'jetblue.com',
  'kenmore air': 'kenmoreair.com',
  klm: 'klm.com',
  'korean air': 'koreanair.com',
  lufthansa: 'lufthansa.com',
  norwegian: 'norwegian.com',
  'norwegian air': 'norwegian.com',
  'qatar airways': 'qatarairways.com',
  qatar: 'qatarairways.com',
  ryanair: 'ryanair.com',
  sas: 'flysas.com',
  'scandinavian airlines': 'flysas.com',
  'singapore airlines': 'singaporeair.com',
  southwest: 'southwest.com',
  'southwest airlines': 'southwest.com',
  swiss: 'swiss.com',
  'swiss international': 'swiss.com',
  tap: 'flytap.com',
  'tap portugal': 'flytap.com',
  'tap air portugal': 'flytap.com',
  'turkish airlines': 'turkishairlines.com',
  united: 'united.com',
  'united airlines': 'united.com',
  'virgin atlantic': 'virginatlantic.com',
  vueling: 'vueling.com',
  'wizz air': 'wizzair.com',
  wizzair: 'wizzair.com',
  eurostar: 'eurostar.com',
  thalys: 'thalys.com',
  sncf: 'sncf.com',
  trenitalia: 'trenitalia.com',
  italo: 'italotreno.it',
  db: 'bahn.de',
  'deutsche bahn': 'bahn.de',
  'intercity express': 'bahn.de',
  obb: 'oebb.at',
  öbb: 'oebb.at',
  sbb: 'sbb.ch',
  cff: 'sbb.ch',
  ffs: 'sbb.ch',
  ns: 'ns.nl',
  'nederlandse spoorwegen': 'ns.nl',
  sj: 'sj.se',
  renfe: 'renfe.com',
  cp: 'cp.pt',
  'comboios de portugal': 'cp.pt',
  'irish rail': 'irishrail.ie',
  'iarnród éireann': 'irishrail.ie',
  'iarnrod eireann': 'irishrail.ie',
  avanti: 'avantiwestcoast.co.uk',
  'avanti west coast': 'avantiwestcoast.co.uk',
  lner: 'lner.co.uk',
  gwr: 'gwr.com',
  'great western railway': 'gwr.com',
  scotrail: 'scotrail.co.uk',
  southeastern: 'southeasternrailway.co.uk',
  northern: 'northernrailway.co.uk',
  'northern trains': 'northernrailway.co.uk',
  crosscountry: 'crosscountrytrains.co.uk',
  transpennine: 'tpexpress.co.uk',
  'transpennine express': 'tpexpress.co.uk',
  'east midlands railway': 'eastmidlandsrailway.co.uk',
  emr: 'eastmidlandsrailway.co.uk',
  amtrak: 'amtrak.com',
  'via rail': 'viarail.ca',
  via: 'viarail.ca',
  korail: 'letskorail.com',
  jr: 'jrpass.com',
  'japan rail': 'jrpass.com',
  dart: 'irishrail.ie',
  regiojet: 'regiojet.com',
  'regiojet train': 'regiojet.com',
  'glacier express': 'glacierexpress.ch',
  'stena line': 'stenaline.com',
  stena: 'stenaline.com',
  'irish ferries': 'irishferries.com',
  'brittany ferries': 'brittany-ferries.co.uk',
  'p&o ferries': 'poferries.com',
  'p&o': 'poferries.com',
  dfds: 'dfds.com',
  'viking line': 'vikingline.com',
  tallink: 'tallink.com',
  'tallink silja': 'tallink.com',
  'color line': 'colorline.com',
  'fjord line': 'fjordline.com',
  'corsica ferries': 'corsica-ferries.co.uk',
  moby: 'moby.it',
  'moby lines': 'moby.it',
  tirrenia: 'tirrenia.it',
  'grimaldi lines': 'grimaldi-lines.com',
  grimaldi: 'grimaldi-lines.com',
  'condor ferries': 'condorferries.co.uk',
  condor: 'condorferries.co.uk',
  wightlink: 'wightlink.co.uk',
  'caledonian macbrayne': 'calmac.co.uk',
  calmac: 'calmac.co.uk',
  flixbus: 'flixbus.com',
  flix: 'flixbus.com',
  greyhound: 'greyhound.com',
  'national express': 'nationalexpress.com',
  megabus: 'megabus.com',
  'bus eireann': 'buseireann.ie',
  'bus éireann': 'buseireann.ie',
  eurolines: 'eurolines.eu',
  ouigo: 'ouigo.com',
};

/**
 * @param {HopResponse} journey
 * @param {number} [size]
 * @returns {string}
 */
function carrierIconHtml(journey, size) {
  const s = size || 20;
  const tt = journey.travel_type || '';
  const fallback = fallbackIcons[tt] || '/static/icons/transport.svg';
  const carrier = journey.carrier || '';
  if (!carrier) {
    return `<img src="${fallback}" alt="${escapeHtml(tt)}" width="${s}" height="${s}" style="vertical-align:middle;border-radius:50%;">`;
  }
  const domain = carrierDomains[carrier.toLowerCase().trim()];
  if (domain) {
    const src = `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    return `<img src="${src}" alt="${escapeHtml(carrier)}" width="${s}" height="${s}" style="vertical-align:middle;border-radius:50%;" onerror="this.onerror=null;this.src='${fallback}';">`;
  }
  return `<img src="${fallback}" alt="${escapeHtml(tt)}" width="${s}" height="${s}" style="vertical-align:middle;border-radius:50%;">`;
}

/**
 * @param {HopResponse} journey
 * @returns {string}
 */
function statusBadgeHtml(journey) {
  if (!journey.status) return '';
  const cssClass = `status-badge status-${escapeHtml(journey.status.toLowerCase().replace(/ /g, '-'))}`;
  const delay = journey.delay_minutes;
  let label = escapeHtml(journey.status);
  if (delay != null && delay > 0) label = `${escapeHtml(journey.status)} (+${delay}m)`;
  else if (delay != null && delay < 0) label = `${escapeHtml(journey.status)} (${delay}m)`;
  return `<span class="${cssClass}">${label}</span>`;
}

/**
 * @param {HopResponse} journey
 * @returns {string}
 */
function verificationBadgeHtml(journey) {
  if (journey.route_verified == null) return '';
  if (journey.route_verified) return '<span class="status-badge status-connected">✓ Verified</span>';
  return '<span class="status-badge status-disconnected">Unverified</span>';
}

/**
 * @param {HopResponse} journey
 * @returns {string}
 */
function platformBadgeHtml(journey) {
  if (journey.travel_type !== 'rail') return '';
  const parts = [];
  if (journey.dep_platform) parts.push(`Pl. ${escapeHtml(journey.dep_platform)}`);
  if (journey.arr_platform) parts.push(`→ Pl. ${escapeHtml(journey.arr_platform)}`);
  if (parts.length === 0) return '';
  return `<span class="platform-badge">${parts.join(' ')}</span>`;
}

/**
 * @param {HopResponse} journey
 * @returns {string}
 */
function journeyCardHtml(journey) {
  const travelTypeKey = journey.travel_type || '';
  const emoji = emojis[travelTypeKey] || '';
  const travelType = escapeHtml(travelTypeKey);
  const typeLabel = travelType.charAt(0).toUpperCase() + travelType.slice(1);
  const originName = escapeHtml(journey.origin_name || '');
  const destName = escapeHtml(journey.dest_name || '');
  const startDate = escapeHtml(journey.start_date || '');
  let dist = '';
  if (
    journey.origin_lat != null &&
    journey.origin_lng != null &&
    journey.dest_lat != null &&
    journey.dest_lng != null
  ) {
    const km = haversineKm(journey.origin_lat, journey.origin_lng, journey.dest_lat, journey.dest_lng);
    dist = km < 1 ? '<1 km' : `${Math.round(km).toLocaleString()} km`;
  }
  return `<a href="/journeys/${journey.id}" class="journey-card-link"><div class="journey-card"><div class="journey-route">${carrierIconHtml(journey, 20)} <span class="journey-origin">${originName}</span><span class="journey-arrow">\u2192</span><span class="journey-dest">${destName}</span></div><div class="journey-meta"><span class="journey-badge badge-${travelType}">${emoji} ${typeLabel}</span>${statusBadgeHtml(journey)}${verificationBadgeHtml(journey)}${platformBadgeHtml(journey)}<span class="journey-date">${startDate}</span>${dist ? `<span class="journey-distance">${dist}</span>` : ''}</div></div></a>`;
}

/**
 * @param {string} dateStr
 * @returns {string}
 */
function countdownText(dateStr) {
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const target = new Date(`${dateStr}T00:00:00`);
  const diffMs = target.getTime() - today.getTime();
  const days = Math.ceil(diffMs / 86400000);
  if (days === 0) return 'Today';
  if (days === 1) return 'Tomorrow';
  return `In ${days} days`;
}

/** @param {HopResponse[]} journeys */
function renderJourneyCards(journeys) {
  const sidebar = document.getElementById('journey-sidebar');
  if (!sidebar) return;
  if (journeys.length === 0) {
    sidebar.innerHTML =
      '<h3 class="journey-sidebar-heading">Journeys</h3><div class="journey-empty">No journeys match the current filters.</div>';
    return;
  }
  const today = new Date().toISOString().slice(0, 10);
  const upcoming = [];
  const past = [];
  journeys.forEach((j) => {
    (j.start_date >= today ? upcoming : past).push(j);
  });
  upcoming.sort((a, b) => a.start_date.localeCompare(b.start_date));
  past.sort((a, b) => b.start_date.localeCompare(a.start_date));

  let html = '';
  if (upcoming.length > 0) {
    html += `<h3 class="journey-sidebar-heading journey-sidebar-heading--upcoming">Upcoming (${upcoming.length})</h3>`;
    upcoming.forEach((j) => {
      html += `<a href="/journeys/${j.id}" class="journey-card-link"><div class="journey-card journey-card--upcoming"><div class="journey-route">${carrierIconHtml(j, 20)} <span class="journey-origin">${escapeHtml(j.origin_name || '')}</span><span class="journey-arrow">\u2192</span><span class="journey-dest">${escapeHtml(j.dest_name || '')}</span></div><div class="journey-meta"><span class="journey-badge badge-${escapeHtml(j.travel_type || '')}">${emojis[j.travel_type] || ''} ${escapeHtml((j.travel_type || '').charAt(0).toUpperCase() + (j.travel_type || '').slice(1))}</span>${statusBadgeHtml(j)}${verificationBadgeHtml(j)}${platformBadgeHtml(j)}<span class="journey-countdown">${countdownText(j.start_date)}</span><span class="journey-date">${escapeHtml(j.start_date || '')}</span></div></div></a>`;
    });
  }
  if (past.length > 0) {
    html += `<h3 class="journey-sidebar-heading">Past Journeys (${past.length})</h3>`;
    past.forEach((j) => { html += journeyCardHtml(j); });
  }
  if (upcoming.length === 0 && past.length === 0) {
    html += '<div class="journey-empty">No journeys match the current filters.</div>';
  }
  sidebar.innerHTML = html;
}

/** @type {maplibregl.Popup} */
const popup = new maplibregl.Popup({ closeButton: true, closeOnClick: true, maxWidth: '360px' });

/** @type {Record<string, {origin_name: string, dest_name: string, from: [number, number], to: [number, number], hops: HopResponse[], freq: number}>} */
let routeIndex = {};

/** @param {HopResponse[]} journeys */
function renderJourneys(journeys) {
  const routes = {};
  const cities = {};
  const bounds = new maplibregl.LngLatBounds();

  journeys.forEach((j) => {
    if ((!j.origin_lat && !j.origin_lng) || (!j.dest_lat && !j.dest_lng)) return;

    const key1 = `${j.origin_name}|${j.origin_lat}|${j.origin_lng}\u2192${j.dest_name}|${j.dest_lat}|${j.dest_lng}`;
    const key2 = `${j.dest_name}|${j.dest_lat}|${j.dest_lng}\u2192${j.origin_name}|${j.origin_lat}|${j.origin_lng}`;
    const key = key1 < key2 ? key1 : key2;
    if (!routes[key]) {
      routes[key] = {
        from: /** @type {[number, number]} */ ([j.origin_lng, j.origin_lat]),
        to: /** @type {[number, number]} */ ([j.dest_lng, j.dest_lat]),
        origin_name: j.origin_name,
        dest_name: j.dest_name,
        hops: [],
      };
    }
    routes[key].hops.push(j);

    const addCity = (/** @type {string} */ name, /** @type {number} */ lat, /** @type {number} */ lng, /** @type {string} */ peer) => {
      const ck = `${name}|${lat}|${lng}`;
      if (!cities[ck]) cities[ck] = { name, lat, lng, count: 0, routes: {} };
      cities[ck].count++;
      cities[ck].routes[peer] = (cities[ck].routes[peer] || 0) + 1;
    };
    if (j.origin_lat != null && j.origin_lng != null) {
      addCity(j.origin_name, j.origin_lat, j.origin_lng, j.dest_name);
      bounds.extend([j.origin_lng, j.origin_lat]);
    }
    if (j.dest_lat != null && j.dest_lng != null) {
      addCity(j.dest_name, j.dest_lat, j.dest_lng, j.origin_name);
      bounds.extend([j.dest_lng, j.dest_lat]);
    }
  });

  const arcFeatures = Object.entries(routes).map(([key, r]) => {
    const freq = r.hops.length;
    const typeCounts = {};
    r.hops.forEach((h) => { typeCounts[h.travel_type] = (typeCounts[h.travel_type] || 0) + 1; });
    const dominantType = Object.keys(typeCounts).sort((a, b) => typeCounts[b] - typeCounts[a])[0];
    const color = typeColors[dominantType] || '#6b7280';
    let width = 1.5;
    if (freq >= 10) width = 3;
    else if (freq >= 5) width = 2.5;
    else if (freq >= 2) width = 2;
    routes[key].freq = freq;
    return {
      type: 'Feature',
      properties: { key, color, width, opacity: Math.min(0.5 + freq * 0.05, 0.9) },
      geometry: { type: 'LineString', coordinates: arcPoints(r.from, r.to, 60) },
    };
  });
  routeIndex = routes;

  const cityFeatures = Object.values(cities).map((/** @type {any} */ c) => ({
    type: 'Feature',
    geometry: { type: 'Point', coordinates: [c.lng, c.lat] },
    properties: { name: c.name, count: c.count, routes: JSON.stringify(c.routes) },
  }));

  if (map.getSource('routes')) {
    /** @type {any} */ (map.getSource('routes')).setData({ type: 'FeatureCollection', features: arcFeatures });
    /** @type {any} */ (map.getSource('cities')).setData({ type: 'FeatureCollection', features: cityFeatures });
  } else {
    map.addSource('routes', { type: 'geojson', data: { type: 'FeatureCollection', features: arcFeatures } });
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

    map.addSource('cities', { type: 'geojson', data: { type: 'FeatureCollection', features: cityFeatures } });
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

    map.on('click', 'route-lines', (e) => {
      if (!e.features || e.features.length === 0) return;
      const key = /** @type {string} */ (e.features[0].properties.key);
      const r = routeIndex[key];
      if (!r) return;
      const dist = haversineKm(r.from[1], r.from[0], r.to[1], r.to[0]);
      const distStr = dist < 1 ? '<1' : Math.round(dist).toLocaleString();
      const sorted = r.hops.slice().sort((a, b) => (b.start_date || '').localeCompare(a.start_date || ''));
      const shown = sorted.slice(0, 8);
      let html = `<div class="journey-popup"><div class="journey-popup-header"><strong>${escapeHtml(r.origin_name || '')} \u2194 ${escapeHtml(r.dest_name || '')}</strong></div><div class="journey-popup-summary"><span>${r.freq} journey${r.freq !== 1 ? 's' : ''}</span><span>\uD83D\uDCCF ${distStr} km</span></div><div class="journey-popup-list">`;
      shown.forEach((j) => {
        const emoji = emojis[j.travel_type] || '';
        const startD = escapeHtml(j.start_date || '');
        const endD = escapeHtml(j.end_date || '');
        const dateStr = startD === endD ? startD : `${startD} \u2192 ${endD}`;
        const journeyOrigin = escapeHtml(j.origin_name || '');
        const routeOrigin = escapeHtml(r.origin_name || '');
        const routeDest = escapeHtml(r.dest_name || '');
        const direction = journeyOrigin === routeOrigin ? `${routeOrigin} \u2192 ${routeDest}` : `${routeDest} \u2192 ${routeOrigin}`;
        html += `<a href="/journeys/${j.id}" class="journey-popup-item"><span class="journey-popup-item-emoji">${emoji}</span><span class="journey-popup-item-direction">${direction}</span><span class="journey-popup-item-date">${dateStr}</span></a>`;
      });
      if (sorted.length > 8) html += `<div class="journey-popup-more">+${sorted.length - 8} more</div>`;
      html += '</div></div>';
      popup.setLngLat(e.lngLat).setHTML(html).addTo(map);
    });

    map.on('click', 'city-circles', (e) => {
      if (!e.features || e.features.length === 0) return;
      const f = e.features[0];
      const name = /** @type {string} */ (f.properties.name);
      const count = /** @type {number} */ (f.properties.count);
      const cityRoutes = JSON.parse(/** @type {string} */ (f.properties.routes));
      const sorted = Object.keys(cityRoutes)
        .map((dest) => ({ name: dest, count: cityRoutes[dest] }))
        .sort((a, b) => b.count - a.count);
      let routeListHtml = sorted
        .slice(0, 5)
        .map((rt) => `<div class="airport-popup-route"><span class="airport-popup-dest">${escapeHtml(rt.name)}</span><span class="airport-popup-freq">${rt.count}\u00d7</span></div>`)
        .join('');
      if (sorted.length > 5) routeListHtml += `<div class="airport-popup-more">+${sorted.length - 5} more destinations</div>`;
      const html = `<div class="airport-popup"><div class="airport-popup-header"><strong>${escapeHtml(name)}</strong></div><div class="airport-popup-stats"><span class="airport-popup-visits">${count} visit${count !== 1 ? 's' : ''}</span><span class="airport-popup-connections">${sorted.length} connection${sorted.length !== 1 ? 's' : ''}</span></div><div class="airport-popup-routes">${routeListHtml}</div></div>`;
      const coords = /** @type {[number, number]} */ (/** @type {any} */ (f.geometry).coordinates);
      popup.setLngLat(coords).setHTML(html).addTo(map);
    });

    map.on('mouseenter', 'route-lines', () => { map.getCanvas().style.cursor = 'pointer'; });
    map.on('mouseleave', 'route-lines', () => { map.getCanvas().style.cursor = ''; });
    map.on('mouseenter', 'city-circles', () => { map.getCanvas().style.cursor = 'pointer'; });
    map.on('mouseleave', 'city-circles', () => { map.getCanvas().style.cursor = ''; });
  }

  document.getElementById('journey-count').textContent = `${journeys.filter((h) => h.origin_lat != null && h.origin_lng != null && h.dest_lat != null && h.dest_lng != null).length} journeys`;

  if (!bounds.isEmpty()) map.fitBounds(bounds, { padding: 40 });
}

const toggleRoutes = document.getElementById('toggle-routes');
const toggleAirports = document.getElementById('toggle-airports');
if (toggleRoutes) {
  toggleRoutes.addEventListener('change', (e) => {
    const checked = /** @type {HTMLInputElement} */ (e.target).checked;
    try { map.setLayoutProperty('route-lines', 'visibility', checked ? 'visible' : 'none'); } catch {}
  });
}
if (toggleAirports) {
  toggleAirports.addEventListener('change', (e) => {
    const checked = /** @type {HTMLInputElement} */ (e.target).checked;
    try { map.setLayoutProperty('city-circles', 'visibility', checked ? 'visible' : 'none'); } catch {}
  });
}

map.on('load', () => {
  renderJourneys(initialJourneys);
  renderJourneyCards(initialJourneys);
});

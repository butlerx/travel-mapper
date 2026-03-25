// @ts-check
/// <reference path="types.d.ts" />

/** @type {HopResponse[]} */
const initialJourneys = JSON.parse(document.getElementById('initial-journeys').textContent || '[]');
/** @type {HopResponse[]} */
let currentJourneys = initialJourneys;
/** @type {ReturnType<typeof setTimeout> | null} */
let debounceTimer = null;

/**
 * @param {string} str
 * @returns {string}
 */
function escapeHtml(str) {
  const div = document.createElement('div');
  div.appendChild(document.createTextNode(str));
  return div.innerHTML;
}

const map = L.map('map', {
  zoomControl: true,
  scrollWheelZoom: true,
  worldCopyJump: true,
  maxBounds: [
    [-85, -Infinity],
    [85, Infinity],
  ],
  maxBoundsViscosity: 1.0,
  minZoom: 2,
}).setView([48, 10], 4);

const darkTiles = L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}@2x.png', {
  attribution:
    '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
  maxZoom: 19,
  subdomains: 'abcd',
});

darkTiles.addTo(map);

const rootStyles = getComputedStyle(document.documentElement);
const cssVar = (name) => {
  return rootStyles.getPropertyValue(name).trim();
};
/** @type {Record<string, string>} */
const colors = {
  air: cssVar('--color-type-air'),
  rail: cssVar('--color-type-rail'),
  boat: cssVar('--color-type-boat'),
  transport: cssVar('--color-type-transport'),
};
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
  // Airlines
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
  // Rail
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
  // Ferry / Boat
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
  // Bus / Transport
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

const filterIds = [
  'search-q',
  'filter-type',
  'filter-origin',
  'filter-dest',
  'filter-date-from',
  'filter-date-to',
  'filter-airline',
  'filter-cabin',
  'filter-reason',
];

/** @type {Record<string, string>} */
const paramMap = {
  'search-q': 'q',
  'filter-type': 'type',
  'filter-origin': 'origin',
  'filter-dest': 'dest',
  'filter-date-from': 'date_from',
  'filter-date-to': 'date_to',
  'filter-airline': 'airline',
  'filter-cabin': 'cabin_class',
  'filter-reason': 'flight_reason',
};

/** @type {Record<string, string>} */
const labelMap = {
  q: 'Search',
  type: 'Type',
  origin: 'Origin',
  dest: 'Dest',
  date_from: 'From',
  date_to: 'To',
  airline: 'Airline',
  cabin_class: 'Cabin',
  flight_reason: 'Reason',
};

const routesLayer = L.layerGroup().addTo(map);
const airportsLayer = L.layerGroup().addTo(map);
const offsets = [-360, 0, 360];

/**
 * @param {number} lat1
 * @param {number} lng1
 * @param {number} lat2
 * @param {number} lng2
 * @returns {number}
 */
function haversineKm(lat1, lng1, lat2, lng2) {
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
 * @param {[number, number]} from
 * @param {[number, number]} to
 * @param {number} numPoints
 * @returns {[number, number][]}
 */
function arcPoints(from, to, numPoints) {
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
    const km = haversineKm(
      journey.origin_lat,
      journey.origin_lng,
      journey.dest_lat,
      journey.dest_lng,
    );
    dist = km < 1 ? '<1 km' : `${Math.round(km).toLocaleString()} km`;
  }

  return `<a href="/journeys/${journey.id}" class="journey-card-link"><div class="journey-card"><div class="journey-route">${carrierIconHtml(journey, 20)} <span class="journey-origin">${originName}</span><span class="journey-arrow">\u2192</span><span class="journey-dest">${destName}</span></div><div class="journey-meta"><span class="journey-badge badge-${travelType}">${emoji} ${typeLabel}</span><span class="journey-date">${startDate}</span>${dist ? `<span class="journey-distance">${dist}</span>` : ''}</div></div></a>`;
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

  journeys.forEach((journey) => {
    if (journey.start_date >= today) {
      upcoming.push(journey);
    } else {
      past.push(journey);
    }
  });

  upcoming.sort((a, b) => {
    return a.start_date.localeCompare(b.start_date);
  });
  past.sort((a, b) => {
    return b.start_date.localeCompare(a.start_date);
  });

  let html = '';

  if (upcoming.length > 0) {
    html += `<h3 class="journey-sidebar-heading journey-sidebar-heading--upcoming">Upcoming (${upcoming.length})</h3>`;
    upcoming.forEach((journey) => {
      html += `<a href="/journeys/${journey.id}" class="journey-card-link"><div class="journey-card journey-card--upcoming"><div class="journey-route">${carrierIconHtml(journey, 20)} <span class="journey-origin">${escapeHtml(journey.origin_name || '')}</span><span class="journey-arrow">\u2192</span><span class="journey-dest">${escapeHtml(journey.dest_name || '')}</span></div><div class="journey-meta"><span class="journey-badge badge-${escapeHtml(journey.travel_type || '')}">${emojis[journey.travel_type] || ''} ${escapeHtml((journey.travel_type || '').charAt(0).toUpperCase() + (journey.travel_type || '').slice(1))}</span><span class="journey-countdown">${countdownText(journey.start_date)}</span><span class="journey-date">${escapeHtml(journey.start_date || '')}</span></div></div></a>`;
    });
  }

  if (past.length > 0) {
    html += `<h3 class="journey-sidebar-heading">Past Journeys (${past.length})</h3>`;
    past.forEach((journey) => {
      html += journeyCardHtml(journey);
    });
  }

  if (upcoming.length === 0 && past.length === 0) {
    html += '<div class="journey-empty">No journeys match the current filters.</div>';
  }

  sidebar.innerHTML = html;
}

/** @param {HopResponse[]} journeys */
function renderJourneys(journeys) {
  routesLayer.clearLayers();
  airportsLayer.clearLayers();
  const bounds = [];

  const routes = {};
  journeys.forEach((journey) => {
    if ((!journey.origin_lat && !journey.origin_lng) || (!journey.dest_lat && !journey.dest_lng))
      return;

    const key1 = `${journey.origin_name}|${journey.origin_lat}|${journey.origin_lng}\u2192${journey.dest_name}|${journey.dest_lat}|${journey.dest_lng}`;
    const key2 = `${journey.dest_name}|${journey.dest_lat}|${journey.dest_lng}\u2192${journey.origin_name}|${journey.origin_lat}|${journey.origin_lng}`;
    const key = key1 < key2 ? key1 : key2;

    if (!routes[key]) {
      routes[key] = {
        from: [journey.origin_lat, journey.origin_lng],
        to: [journey.dest_lat, journey.dest_lng],
        origin_name: journey.origin_name,
        dest_name: journey.dest_name,
        hops: [],
      };
    }
    routes[key].hops.push(journey);
  });

  Object.keys(routes).forEach((key) => {
    const route = routes[key];
    const from = route.from;
    const to = route.to;
    const routeJourneys = route.hops;
    const freq = routeJourneys.length;

    const typeCounts = {};
    routeJourneys.forEach((h) => {
      typeCounts[h.travel_type] = (typeCounts[h.travel_type] || 0) + 1;
    });
    const dominantType = Object.keys(typeCounts).sort((a, b) => {
      return typeCounts[b] - typeCounts[a];
    })[0];
    const color = colors[dominantType] || '#6b7280';

    const points = arcPoints(from, to, 50);
    const dist = haversineKm(from[0], from[1], to[0], to[1]);
    const distStr = dist < 1 ? '<1' : Math.round(dist).toLocaleString();

    let weight = 2;
    let opacity = 0.6;
    if (freq >= 10) {
      weight = 5.5;
      opacity = 0.9;
    } else if (freq >= 5) {
      weight = 4;
      opacity = 0.8;
    } else if (freq >= 2) {
      weight = 3;
      opacity = 0.7;
    }

    let popup = `<div class="journey-popup"><div class="journey-popup-header"><strong>${escapeHtml(route.origin_name || '')} \u2194 ${escapeHtml(route.dest_name || '')}</strong></div><div class="journey-popup-summary"><span>${freq} journey${freq !== 1 ? 's' : ''}</span><span>\uD83D\uDCCF ${distStr} km</span></div><div class="journey-popup-list">`;

    const sorted = routeJourneys.slice().sort((a, b) => {
      return (b.start_date || '').localeCompare(a.start_date || '');
    });
    const shown = sorted.slice(0, 8);
    shown.forEach((journey) => {
      const emoji = emojis[journey.travel_type] || '';
      const startD = escapeHtml(journey.start_date || '');
      const endD = escapeHtml(journey.end_date || '');
      const dateStr = startD === endD ? startD : `${startD} \u2192 ${endD}`;
      const routeOriginName = escapeHtml(route.origin_name || '');
      const routeDestName = escapeHtml(route.dest_name || '');
      const journeyOriginName = escapeHtml(journey.origin_name || '');
      const direction =
        journeyOriginName === routeOriginName
          ? `${routeOriginName} \u2192 ${routeDestName}`
          : `${routeDestName} \u2192 ${routeOriginName}`;

      popup += `<a href="/journeys/${journey.id}" class="journey-popup-item"><span class="journey-popup-item-emoji">${emoji}</span><span class="journey-popup-item-direction">${direction}</span><span class="journey-popup-item-date">${dateStr}</span></a>`;
    });
    if (sorted.length > 8) {
      popup += `<div class="journey-popup-more">+${sorted.length - 8} more</div>`;
    }
    popup += '</div></div>';

    offsets.forEach((offset) => {
      const shifted = /** @type {[number, number][]} */ (
        points.map((p) => {
          return [p[0], p[1] + offset];
        })
      );
      L.polyline(shifted, {
        color,
        weight,
        opacity,
      })
        .bindPopup(popup, { maxWidth: 360, className: 'journey-popup-container' })
        .addTo(routesLayer);
    });

    bounds.push(from);
    bounds.push(to);
  });

  const cities = {};
  journeys.forEach((journey) => {
    if (journey.origin_lat != null && journey.origin_lng != null) {
      const oKey = `${journey.origin_name}|${journey.origin_lat}|${journey.origin_lng}`;
      if (!cities[oKey]) {
        cities[oKey] = {
          name: journey.origin_name,
          lat: journey.origin_lat,
          lng: journey.origin_lng,
          count: 0,
          routes: {},
        };
      }
      cities[oKey].count++;
      cities[oKey].routes[journey.dest_name] = (cities[oKey].routes[journey.dest_name] || 0) + 1;
    }
    if (journey.dest_lat != null && journey.dest_lng != null) {
      const dKey = `${journey.dest_name}|${journey.dest_lat}|${journey.dest_lng}`;
      if (!cities[dKey]) {
        cities[dKey] = {
          name: journey.dest_name,
          lat: journey.dest_lat,
          lng: journey.dest_lng,
          count: 0,
          routes: {},
        };
      }
      cities[dKey].count++;
      cities[dKey].routes[journey.origin_name] =
        (cities[dKey].routes[journey.origin_name] || 0) + 1;
    }
  });

  Object.keys(cities).forEach((key) => {
    const c = cities[key];
    const r = Math.max(4, Math.min(8, 3 + Math.sqrt(c.count)));
    const markerOpts = {
      radius: r,
      color: '#1e3a5f',
      weight: 1.5,
      fillColor: cssVar('--color-type-air'),
      fillOpacity: 0.85,
    };
    const sortedRoutes = Object.keys(c.routes)
      .map((dest) => {
        return { name: dest, count: c.routes[dest] };
      })
      .sort((a, b) => {
        return b.count - a.count;
      });
    let routeListHtml = sortedRoutes
      .slice(0, 5)
      .map((rt) => {
        return `<div class="airport-popup-route"><span class="airport-popup-dest">${escapeHtml(rt.name || '')}</span><span class="airport-popup-freq">${rt.count}\u00d7</span></div>`;
      })
      .join('');
    if (sortedRoutes.length > 5) {
      routeListHtml += `<div class="airport-popup-more">+${sortedRoutes.length - 5} more destinations</div>`;
    }
    const popupHtml = `<div class="airport-popup"><div class="airport-popup-header"><strong>${escapeHtml(c.name || '')}</strong></div><div class="airport-popup-stats"><span class="airport-popup-visits">${c.count} visit${c.count !== 1 ? 's' : ''}</span><span class="airport-popup-connections">${sortedRoutes.length} connection${sortedRoutes.length !== 1 ? 's' : ''}</span></div><div class="airport-popup-routes">${routeListHtml}</div></div>`;

    offsets.forEach((offset) => {
      const marker = L.circleMarker([c.lat, c.lng + offset], markerOpts).bindPopup(popupHtml, {
        maxWidth: 280,
        className: 'airport-popup-container',
      });
      marker.on('mouseover', function () {
        marker.openPopup();
      });
      marker.on('mouseout', function () {
        if (!marker._popupHandlingClick) {
          marker.closePopup();
        }
      });
      marker.on('click', function () {
        marker._popupHandlingClick = true;
        marker.openPopup();
      });
      marker.on('popupclose', function () {
        marker._popupHandlingClick = false;
      });
      marker.addTo(airportsLayer);
    });
  });

  document.getElementById('journey-count').textContent = `${
    journeys.filter((h) => {
      return (
        h.origin_lat != null && h.origin_lng != null && h.dest_lat != null && h.dest_lng != null
      );
    }).length
  } journeys`;

  if (bounds.length > 0) {
    map.fitBounds(bounds, { padding: [40, 40] });
  }
}

/** @returns {FilterParams} */
function getFilterValues() {
  const params = {};
  filterIds.forEach((id) => {
    const el = /** @type {HTMLInputElement | null} */ (document.getElementById(id));
    if (el && el.value) {
      params[paramMap[id]] = el.value;
    }
  });
  return params;
}

/**
 * @param {FilterParams} params
 * @returns {boolean}
 */
function hasActiveFilters(params) {
  return Object.keys(params).length > 0;
}

/**
 * @param {FilterParams} params
 * @returns {string}
 */
function buildQueryString(params) {
  const parts = [];
  Object.keys(params).forEach((key) => {
    if (params[key]) {
      parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(params[key])}`);
    }
  });
  return parts.length > 0 ? `?${parts.join('&')}` : '';
}

/** @param {FilterParams} params */
function syncUrlParams(params) {
  const qs = buildQueryString(params);
  const newUrl = `${window.location.pathname}${qs}`;
  history.replaceState(null, '', newUrl);
}

/** @param {FilterParams} params */
function renderActiveFilters(params) {
  const container = document.getElementById('active-filters');
  if (!container) return;

  if (!hasActiveFilters(params)) {
    container.innerHTML = '';
    return;
  }

  let html = '';
  Object.keys(params).forEach((key) => {
    if (params[key]) {
      const label = labelMap[key] || key;
      const safeLabel = escapeHtml(label);
      const safeValue = escapeHtml(params[key]);
      html += `<span class="filter-chip" data-param="${key}"><span class="filter-chip-label">${safeLabel}:</span> ${safeValue}<button type="button" class="filter-chip-remove" aria-label="Remove ${safeLabel}">\u00d7</button></span>`;
    }
  });
  container.innerHTML = html;

  container.querySelectorAll('.filter-chip-remove').forEach((btn) => {
    btn.addEventListener('click', () => {
      const chip = btn.closest('.filter-chip');
      const paramKey = chip.getAttribute('data-param');
      const reverseMap = {};
      Object.keys(paramMap).forEach((id) => {
        reverseMap[paramMap[id]] = id;
      });
      const inputId = reverseMap[paramKey];
      if (inputId) {
        const el = /** @type {HTMLInputElement | null} */ (document.getElementById(inputId));
        if (el) {
          el.value = '';
        }
      }
      applyFilters();
    });
  });
}

/** @param {FilterParams} params */
async function fetchAndRender(params) {
  const qs = buildQueryString(params);
  try {
    const res = await fetch(`/journeys${qs}`, {
      headers: { Accept: 'application/json' },
      credentials: 'same-origin',
    });
    if (!res.ok) throw new Error('Failed to fetch journeys');
    const journeys = await res.json();
    currentJourneys = journeys;
    renderJourneys(journeys);
    renderJourneyCards(journeys);
  } catch {
    currentJourneys = [];
    renderJourneys([]);
    renderJourneyCards([]);
  }
}

function applyFilters() {
  const params = getFilterValues();
  syncUrlParams(params);
  renderActiveFilters(params);

  if (hasActiveFilters(params)) {
    fetchAndRender(params);
  } else {
    currentJourneys = initialJourneys;
    renderJourneys(initialJourneys);
    renderJourneyCards(initialJourneys);
  }
}

function debouncedApply() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(applyFilters, 300);
}

/** @returns {boolean} */
function populateFromUrl() {
  const urlParams = new URLSearchParams(window.location.search);
  const reverseMap = {};
  Object.keys(paramMap).forEach((id) => {
    reverseMap[paramMap[id]] = id;
  });

  let hasParams = false;
  urlParams.forEach((value, key) => {
    const inputId = reverseMap[key];
    if (inputId) {
      const el = /** @type {HTMLInputElement | null} */ (document.getElementById(inputId));
      if (el) {
        el.value = value;
        hasParams = true;
      }
    }
  });

  return hasParams;
}

function clearFilters() {
  filterIds.forEach((id) => {
    const el = /** @type {HTMLInputElement | null} */ (document.getElementById(id));
    if (el) el.value = '';
  });
  syncUrlParams({});
  renderActiveFilters({});
  currentJourneys = initialJourneys;
  renderJourneys(initialJourneys);
  renderJourneyCards(initialJourneys);
}

const textInputs = ['search-q', 'filter-origin', 'filter-dest', 'filter-airline'];
const selectInputs = ['filter-type', 'filter-cabin', 'filter-reason'];
const dateInputs = ['filter-date-from', 'filter-date-to'];

textInputs.forEach((id) => {
  const el = document.getElementById(id);
  if (el) el.addEventListener('input', debouncedApply);
});

selectInputs.forEach((id) => {
  const el = document.getElementById(id);
  if (el) el.addEventListener('change', applyFilters);
});

dateInputs.forEach((id) => {
  const el = document.getElementById(id);
  if (el) el.addEventListener('change', applyFilters);
});

const clearBtn = document.getElementById('filter-clear');
if (clearBtn) clearBtn.addEventListener('click', clearFilters);

const toggleRoutes = document.getElementById('toggle-routes');
const toggleAirports = document.getElementById('toggle-airports');
if (toggleRoutes) {
  toggleRoutes.addEventListener('change', (event) => {
    const target = /** @type {HTMLInputElement} */ (event.target);
    if (target.checked) {
      map.addLayer(routesLayer);
    } else {
      map.removeLayer(routesLayer);
    }
  });
}
if (toggleAirports) {
  toggleAirports.addEventListener('change', (event) => {
    const target = /** @type {HTMLInputElement} */ (event.target);
    if (target.checked) {
      map.addLayer(airportsLayer);
    } else {
      map.removeLayer(airportsLayer);
    }
  });
}

const hasUrlFilters = populateFromUrl();
if (hasUrlFilters) {
  const params = getFilterValues();
  renderActiveFilters(params);
  fetchAndRender(params);
} else {
  renderJourneys(initialJourneys);
  renderJourneyCards(initialJourneys);
}

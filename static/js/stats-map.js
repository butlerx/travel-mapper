// @ts-check
/// <reference path="types.d.ts" />
import { createMap, escapeHtml } from './map-core.js';

/** @type {CountryCounts} */
const counts = JSON.parse(document.getElementById('country-counts').textContent || '{}');
const mapEl = document.getElementById('stats-map');
if (!mapEl || Object.keys(counts).length === 0) {
} else {
  const map = createMap('stats-map', { center: [10, 30], zoom: 2, scrollZoom: false });

  const n2a = {
    '004': 'AF', '008': 'AL', '010': 'AQ', '012': 'DZ', '016': 'AS', '020': 'AD',
    '024': 'AO', '028': 'AG', '031': 'AZ', '032': 'AR', '036': 'AU', '040': 'AT',
    '044': 'BS', '048': 'BH', '050': 'BD', '051': 'AM', '052': 'BB', '056': 'BE',
    '060': 'BM', '064': 'BT', '068': 'BO', '070': 'BA', '072': 'BW', '076': 'BR',
    '084': 'BZ', '090': 'SB', '096': 'BN', 100: 'BG', 104: 'MM', 108: 'BI',
    112: 'BY', 116: 'KH', 120: 'CM', 124: 'CA', 132: 'CV', 140: 'CF', 144: 'LK',
    148: 'TD', 152: 'CL', 156: 'CN', 158: 'TW', 170: 'CO', 174: 'KM', 175: 'YT',
    178: 'CG', 180: 'CD', 184: 'CK', 188: 'CR', 191: 'HR', 192: 'CU', 196: 'CY',
    203: 'CZ', 204: 'BJ', 208: 'DK', 212: 'DM', 214: 'DO', 218: 'EC', 222: 'SV',
    226: 'GQ', 231: 'ET', 232: 'ER', 233: 'EE', 234: 'FO', 238: 'FK', 242: 'FJ',
    246: 'FI', 250: 'FR', 254: 'GF', 258: 'PF', 260: 'TF', 262: 'DJ', 266: 'GA',
    268: 'GE', 270: 'GM', 275: 'PS', 276: 'DE', 288: 'GH', 292: 'GI', 296: 'KI',
    300: 'GR', 304: 'GL', 308: 'GD', 312: 'GP', 316: 'GU', 320: 'GT', 324: 'GN',
    328: 'GY', 332: 'HT', 336: 'VA', 340: 'HN', 344: 'HK', 348: 'HU', 352: 'IS',
    356: 'IN', 360: 'ID', 364: 'IR', 368: 'IQ', 372: 'IE', 376: 'IL', 380: 'IT',
    384: 'CI', 388: 'JM', 392: 'JP', 398: 'KZ', 400: 'JO', 404: 'KE', 408: 'KP',
    410: 'KR', 414: 'KW', 417: 'KG', 418: 'LA', 422: 'LB', 426: 'LS', 428: 'LV',
    430: 'LR', 434: 'LY', 438: 'LI', 440: 'LT', 442: 'LU', 446: 'MO', 450: 'MG',
    454: 'MW', 458: 'MY', 462: 'MV', 466: 'ML', 470: 'MT', 474: 'MQ', 478: 'MR',
    480: 'MU', 484: 'MX', 492: 'MC', 496: 'MN', 498: 'MD', 499: 'ME', 504: 'MA',
    508: 'MZ', 512: 'OM', 516: 'NA', 520: 'NR', 524: 'NP', 528: 'NL', 530: 'AN',
    533: 'AW', 540: 'NC', 548: 'VU', 554: 'NZ', 558: 'NI', 562: 'NE', 566: 'NG',
    570: 'NU', 574: 'NF', 578: 'NO', 580: 'MP', 583: 'FM', 584: 'MH', 585: 'PW',
    586: 'PK', 591: 'PA', 598: 'PG', 600: 'PY', 604: 'PE', 608: 'PH', 612: 'PN',
    616: 'PL', 620: 'PT', 624: 'GW', 626: 'TL', 630: 'PR', 634: 'QA', 638: 'RE',
    642: 'RO', 643: 'RU', 646: 'RW', 654: 'SH', 659: 'KN', 660: 'AI', 662: 'LC',
    666: 'PM', 670: 'VC', 674: 'SM', 678: 'ST', 682: 'SA', 686: 'SN', 688: 'RS',
    690: 'SC', 694: 'SL', 702: 'SG', 703: 'SK', 704: 'VN', 705: 'SI', 706: 'SO',
    710: 'ZA', 716: 'ZW', 720: 'YE', 724: 'ES', 732: 'EH', 736: 'SD', 740: 'SR',
    744: 'SJ', 748: 'SZ', 752: 'SE', 756: 'CH', 760: 'SY', 762: 'TJ', 764: 'TH',
    768: 'TG', 772: 'TK', 776: 'TO', 780: 'TT', 784: 'AE', 788: 'TN', 792: 'TR',
    795: 'TM', 796: 'TC', 798: 'TV', 800: 'UG', 804: 'UA', 807: 'MK', 818: 'EG',
    826: 'GB', 834: 'TZ', 840: 'US', 854: 'BF', 858: 'UY', 860: 'UZ', 862: 'VE',
    876: 'WF', 882: 'WS', 887: 'YE', 894: 'ZM', '-99': 'CY', 900: 'XK',
  };

  /**
   * @param {number} count
   * @returns {string}
   */
  function getColor(count) {
    if (count >= 50) return '#fde724';
    if (count >= 25) return '#6ece58';
    if (count >= 15) return '#1f9e89';
    if (count >= 10) return '#26828e';
    if (count >= 5) return '#31688e';
    if (count >= 3) return '#3e4989';
    if (count >= 2) return '#482878';
    if (count >= 1) return '#473677';
    return 'transparent';
  }

  // Tooltip overlay
  const tooltipEl = document.createElement('div');
  tooltipEl.className = 'stats-map-tooltip';
  tooltipEl.style.display = 'none';
  mapEl.appendChild(tooltipEl);

  // Legend
  const legendEl = document.createElement('div');
  legendEl.className = 'stats-map-legend';
  const grades = [1, 2, 3, 5, 10, 15, 25, 50];
  const labels = ['1', '2', '3\u20134', '5\u20139', '10\u201314', '15\u201324', '25\u201349', '50+'];
  legendEl.innerHTML = '<strong>Visits</strong>' +
    grades.map((g, i) => `<div class="stats-map-legend-row"><span class="stats-map-legend-swatch" style="background:${getColor(g)}"></span>${labels[i]}</div>`).join('');
  mapEl.appendChild(legendEl);

  map.on('load', async () => {
    try {
      const r = await fetch('https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json');
      const topo = await r.json();
      const geo = topojson.feature(topo, topo.objects.countries);

      // Inject visit_count into each feature's properties
      geo.features.forEach((f) => {
        const id = f.id || (f.properties && f.properties.id);
        const alpha2 = n2a[String(id)] || '';
        f.properties = f.properties || {};
        f.properties.visit_count = counts[alpha2] || 0;
        f.properties.name = (f.properties && f.properties.name) || alpha2 || 'Unknown';
      });

      map.addSource('countries', { type: 'geojson', data: geo });

      map.addLayer({
        id: 'countries-fill',
        type: 'fill',
        source: 'countries',
        paint: {
          'fill-color': [
            'step', ['get', 'visit_count'],
            'transparent',
            1, '#473677',
            2, '#482878',
            3, '#3e4989',
            5, '#31688e',
            10, '#26828e',
            15, '#1f9e89',
            25, '#6ece58',
            50, '#fde724',
          ],
          'fill-opacity': ['case', ['>', ['get', 'visit_count'], 0], 0.75, 0],
        },
      });

      map.addLayer({
        id: 'countries-outline',
        type: 'line',
        source: 'countries',
        paint: {
          'line-color': ['case', ['>', ['get', 'visit_count'], 0], 'rgba(255,255,255,0.4)', 'transparent'],
          'line-width': ['case', ['>', ['get', 'visit_count'], 0], 0.8, 0],
        },
      });

      map.on('mousemove', 'countries-fill', (e) => {
        if (!e.features || e.features.length === 0) return;
        const f = e.features[0];
        const name = /** @type {string} */ (f.properties.name);
        const count = /** @type {number} */ (f.properties.visit_count);
        tooltipEl.innerHTML = `<strong>${escapeHtml(name)}</strong><br>${count} visit${count === 1 ? '' : 's'}`;
        tooltipEl.style.display = 'block';
        map.getCanvas().style.cursor = count > 0 ? 'pointer' : '';

        if (count > 0) {
          map.setPaintProperty('countries-outline', 'line-width', [
            'case', ['==', ['id'], f.id], 2, ['>', ['get', 'visit_count'], 0], 0.8, 0,
          ]);
        }
      });

      map.on('mouseleave', 'countries-fill', () => {
        tooltipEl.style.display = 'none';
        map.getCanvas().style.cursor = '';
        map.setPaintProperty('countries-outline', 'line-width', [
          'case', ['>', ['get', 'visit_count'], 0], 0.8, 0,
        ]);
      });
    } catch {}
  });
}

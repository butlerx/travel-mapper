(function () {
  var counts = window.countryCounts || {};
  var mapEl = document.getElementById('stats-map');
  if (!mapEl || Object.keys(counts).length === 0) return;

  var map = L.map('stats-map', {
    zoomControl: true,
    scrollWheelZoom: false,
    worldCopyJump: true,
    maxBounds: [
      [-85, -Infinity],
      [85, Infinity],
    ],
    maxBoundsViscosity: 1.0,
    minZoom: 2,
  }).setView([30, 10], 2);

  L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}@2x.png', {
    attribution:
      '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
    maxZoom: 19,
    subdomains: 'abcd',
  }).addTo(map);

  // ISO 3166-1 numeric → alpha-2 lookup (world-atlas uses numeric IDs)
  var n2a = {
    '004': 'AF',
    '008': 'AL',
    '010': 'AQ',
    '012': 'DZ',
    '016': 'AS',
    '020': 'AD',
    '024': 'AO',
    '028': 'AG',
    '031': 'AZ',
    '032': 'AR',
    '036': 'AU',
    '040': 'AT',
    '044': 'BS',
    '048': 'BH',
    '050': 'BD',
    '051': 'AM',
    '052': 'BB',
    '056': 'BE',
    '060': 'BM',
    '064': 'BT',
    '068': 'BO',
    '070': 'BA',
    '072': 'BW',
    '076': 'BR',
    '084': 'BZ',
    '090': 'SB',
    '096': 'BN',
    100: 'BG',
    104: 'MM',
    108: 'BI',
    112: 'BY',
    116: 'KH',
    120: 'CM',
    124: 'CA',
    132: 'CV',
    140: 'CF',
    144: 'LK',
    148: 'TD',
    152: 'CL',
    156: 'CN',
    158: 'TW',
    170: 'CO',
    174: 'KM',
    175: 'YT',
    178: 'CG',
    180: 'CD',
    184: 'CK',
    188: 'CR',
    191: 'HR',
    192: 'CU',
    196: 'CY',
    203: 'CZ',
    204: 'BJ',
    208: 'DK',
    212: 'DM',
    214: 'DO',
    218: 'EC',
    222: 'SV',
    226: 'GQ',
    231: 'ET',
    232: 'ER',
    233: 'EE',
    234: 'FO',
    238: 'FK',
    242: 'FJ',
    246: 'FI',
    250: 'FR',
    254: 'GF',
    258: 'PF',
    260: 'TF',
    262: 'DJ',
    266: 'GA',
    268: 'GE',
    270: 'GM',
    275: 'PS',
    276: 'DE',
    288: 'GH',
    292: 'GI',
    296: 'KI',
    300: 'GR',
    304: 'GL',
    308: 'GD',
    312: 'GP',
    316: 'GU',
    320: 'GT',
    324: 'GN',
    328: 'GY',
    332: 'HT',
    336: 'VA',
    340: 'HN',
    344: 'HK',
    348: 'HU',
    352: 'IS',
    356: 'IN',
    360: 'ID',
    364: 'IR',
    368: 'IQ',
    372: 'IE',
    376: 'IL',
    380: 'IT',
    384: 'CI',
    388: 'JM',
    392: 'JP',
    398: 'KZ',
    400: 'JO',
    404: 'KE',
    408: 'KP',
    410: 'KR',
    414: 'KW',
    417: 'KG',
    418: 'LA',
    422: 'LB',
    426: 'LS',
    428: 'LV',
    430: 'LR',
    434: 'LY',
    438: 'LI',
    440: 'LT',
    442: 'LU',
    446: 'MO',
    450: 'MG',
    454: 'MW',
    458: 'MY',
    462: 'MV',
    466: 'ML',
    470: 'MT',
    474: 'MQ',
    478: 'MR',
    480: 'MU',
    484: 'MX',
    492: 'MC',
    496: 'MN',
    498: 'MD',
    499: 'ME',
    504: 'MA',
    508: 'MZ',
    512: 'OM',
    516: 'NA',
    520: 'NR',
    524: 'NP',
    528: 'NL',
    530: 'AN',
    533: 'AW',
    540: 'NC',
    548: 'VU',
    554: 'NZ',
    558: 'NI',
    562: 'NE',
    566: 'NG',
    570: 'NU',
    574: 'NF',
    578: 'NO',
    580: 'MP',
    583: 'FM',
    584: 'MH',
    585: 'PW',
    586: 'PK',
    591: 'PA',
    598: 'PG',
    600: 'PY',
    604: 'PE',
    608: 'PH',
    612: 'PN',
    616: 'PL',
    620: 'PT',
    624: 'GW',
    626: 'TL',
    630: 'PR',
    634: 'QA',
    638: 'RE',
    642: 'RO',
    643: 'RU',
    646: 'RW',
    654: 'SH',
    659: 'KN',
    660: 'AI',
    662: 'LC',
    666: 'PM',
    670: 'VC',
    674: 'SM',
    678: 'ST',
    682: 'SA',
    686: 'SN',
    688: 'RS',
    690: 'SC',
    694: 'SL',
    702: 'SG',
    703: 'SK',
    704: 'VN',
    705: 'SI',
    706: 'SO',
    710: 'ZA',
    716: 'ZW',
    720: 'YE',
    724: 'ES',
    732: 'EH',
    736: 'SD',
    740: 'SR',
    744: 'SJ',
    748: 'SZ',
    752: 'SE',
    756: 'CH',
    760: 'SY',
    762: 'TJ',
    764: 'TH',
    768: 'TG',
    772: 'TK',
    776: 'TO',
    780: 'TT',
    784: 'AE',
    788: 'TN',
    792: 'TR',
    795: 'TM',
    796: 'TC',
    798: 'TV',
    800: 'UG',
    804: 'UA',
    807: 'MK',
    818: 'EG',
    826: 'GB',
    834: 'TZ',
    840: 'US',
    854: 'BF',
    858: 'UY',
    860: 'UZ',
    862: 'VE',
    876: 'WF',
    882: 'WS',
    887: 'YE',
    894: 'ZM',
    '-99': 'CY',
    '010': 'AQ',
    '070': 'BA',
    688: 'RS',
    499: 'ME',
    900: 'XK',
  };

  // Viridis palette — perceptually uniform, colorblind-safe
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

  // Fix antimeridian rendering — polygons crossing 180° longitude
  // (e.g. Russia, Fiji) get a horizontal line artifact in Leaflet.
  // Shift negative longitudes to >180 so the polygon doesn't wrap.
  function fixAntimeridian(coords) {
    var dominated = 0;
    var i, j;
    // Check if the majority of points are in the eastern hemisphere
    for (i = 0; i < coords.length; i++) {
      for (j = 0; j < coords[i].length; j++) {
        if (Array.isArray(coords[i][j][0])) {
          // MultiPolygon ring
          for (var k = 0; k < coords[i][j].length; k++) {
            if (coords[i][j][k][0] > 0) dominated++;
            else dominated--;
          }
        } else {
          if (coords[i][j][0] > 0) dominated++;
          else dominated--;
        }
      }
    }
    if (dominated <= 0) return;
    // Shift negative longitudes for eastern-dominated polygons
    for (i = 0; i < coords.length; i++) {
      for (j = 0; j < coords[i].length; j++) {
        if (Array.isArray(coords[i][j][0])) {
          for (var m = 0; m < coords[i][j].length; m++) {
            if (coords[i][j][m][0] < 0) coords[i][j][m][0] += 360;
          }
        } else {
          if (coords[i][j][0] < 0) coords[i][j][0] += 360;
        }
      }
    }
  }

  function needsAntimeridianFix(feature) {
    var coords = feature.geometry.coordinates;
    if (!coords) return false;
    // Check if any ring spans > 180° longitude
    function checkRing(ring) {
      var minLng = Infinity,
        maxLng = -Infinity;
      for (var i = 0; i < ring.length; i++) {
        var lng = ring[i][0];
        if (lng < minLng) minLng = lng;
        if (lng > maxLng) maxLng = lng;
      }
      return maxLng - minLng > 180;
    }
    function checkPolygon(poly) {
      for (var i = 0; i < poly.length; i++) {
        if (checkRing(poly[i])) return true;
      }
      return false;
    }
    var type = feature.geometry.type;
    if (type === 'Polygon') return checkPolygon(coords);
    if (type === 'MultiPolygon') {
      for (var p = 0; p < coords.length; p++) {
        if (checkPolygon(coords[p])) return true;
      }
    }
    return false;
  }

  function style(feature) {
    var id = feature.id || (feature.properties && feature.properties.id);
    var alpha2 = n2a[String(id)] || '';
    var count = counts[alpha2] || 0;
    return {
      fillColor: getColor(count),
      fillOpacity: count > 0 ? 0.75 : 0,
      color: count > 0 ? 'rgba(255,255,255,0.4)' : 'transparent',
      weight: count > 0 ? 0.8 : 0,
    };
  }

  var info = L.control({ position: 'topright' });
  info.onAdd = function () {
    this._div = L.DomUtil.create('div', 'stats-map-tooltip');
    this.update();
    return this._div;
  };
  info.update = function (name, count) {
    if (name) {
      this._div.innerHTML =
        '<strong>' + name + '</strong><br>' + (count || 0) + ' visit' + (count === 1 ? '' : 's');
      this._div.style.display = 'block';
    } else {
      this._div.style.display = 'none';
    }
  };
  info.addTo(map);

  var legend = L.control({ position: 'bottomright' });
  legend.onAdd = function () {
    var div = L.DomUtil.create('div', 'stats-map-legend');
    var grades = [1, 2, 3, 5, 10, 15, 25, 50];
    var labels = [
      '1',
      '2',
      '3\u20134',
      '5\u20139',
      '10\u201314',
      '15\u201324',
      '25\u201349',
      '50+',
    ];
    div.innerHTML = '<strong>Visits</strong>';
    for (var i = 0; i < grades.length; i++) {
      div.innerHTML +=
        '<div class="stats-map-legend-row">' +
        '<span class="stats-map-legend-swatch" style="background:' +
        getColor(grades[i]) +
        '"></span>' +
        labels[i] +
        '</div>';
    }
    return div;
  };
  legend.addTo(map);

  fetch('https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json')
    .then(function (r) {
      return r.json();
    })
    .then(function (topo) {
      var geo = topojson.feature(topo, topo.objects.countries);
      geo.features.forEach(function (f) {
        if (needsAntimeridianFix(f)) fixAntimeridian(f.geometry.coordinates);
      });
      L.geoJSON(geo, {
        style: style,
        onEachFeature: function (feature, layer) {
          var id = feature.id || (feature.properties && feature.properties.id);
          var alpha2 = n2a[String(id)] || '';
          var count = counts[alpha2] || 0;
          var name = (feature.properties && feature.properties.name) || alpha2 || 'Unknown';
          layer.on({
            mouseover: function (e) {
              var l = e.target;
              if (count > 0) {
                l.setStyle({ weight: 2, color: 'rgba(255,255,255,0.7)', fillOpacity: 0.9 });
                if (!L.Browser.ie && !L.Browser.opera && !L.Browser.edge) {
                  l.bringToFront();
                }
              }
              info.update(name, count);
            },
            mouseout: function (e) {
              var l = e.target;
              l.setStyle(style(feature));
              info.update();
            },
          });
        },
      }).addTo(map);
    });
})();

(function () {
  var allHops = window.allHops || [];
  var isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

  var map = L.map('map', {
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

  var lightTiles = L.tileLayer('https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}@2x.png', {
    attribution:
      '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
    maxZoom: 19,
    subdomains: 'abcd',
  });
  var darkTiles = L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}@2x.png', {
    attribution:
      '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
    maxZoom: 19,
    subdomains: 'abcd',
  });

  if (isDark) darkTiles.addTo(map);
  else lightTiles.addTo(map);

  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function (e) {
    isDark = e.matches;
    if (e.matches) {
      map.removeLayer(lightTiles);
      darkTiles.addTo(map);
    } else {
      map.removeLayer(darkTiles);
      lightTiles.addTo(map);
    }
    applyFilters();
  });

  var colors = {
    air: '#0077bb',
    rail: '#ee7733',
    cruise: '#cc3311',
    transport: '#009988',
  };
  var emojis = {
    air: '\u2708\uFE0F',
    rail: '\uD83D\uDE86',
    cruise: '\uD83D\uDEA2',
    transport: '\uD83D\uDE97',
  };

  var years = [];
  allHops.forEach(function (h) {
    var y = h.start_date.substring(0, 4);
    if (y && years.indexOf(y) === -1) years.push(y);
  });
  years.sort().reverse();
  var yearSelect = document.getElementById('filter-year');
  years.forEach(function (y) {
    var opt = document.createElement('option');
    opt.value = y;
    opt.textContent = y;
    yearSelect.appendChild(opt);
  });

  var routeLayer = L.layerGroup().addTo(map);
  var offsets = [-360, 0, 360];

  function haversineKm(lat1, lng1, lat2, lng2) {
    var R = 6371;
    var dLat = ((lat2 - lat1) * Math.PI) / 180;
    var dLng = ((lng2 - lng1) * Math.PI) / 180;
    var a =
      Math.sin(dLat / 2) * Math.sin(dLat / 2) +
      Math.cos((lat1 * Math.PI) / 180) *
        Math.cos((lat2 * Math.PI) / 180) *
        Math.sin(dLng / 2) *
        Math.sin(dLng / 2);
    return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  }

  function arcPoints(from, to, numPoints) {
    var lat1 = (from[0] * Math.PI) / 180;
    var lng1 = (from[1] * Math.PI) / 180;
    var lat2 = (to[0] * Math.PI) / 180;
    var lng2 = (to[1] * Math.PI) / 180;

    var dLng = lng2 - lng1;
    if (dLng > Math.PI) lng2 -= 2 * Math.PI;
    else if (dLng < -Math.PI) lng2 += 2 * Math.PI;

    var d =
      2 *
      Math.asin(
        Math.sqrt(
          Math.pow(Math.sin((lat1 - lat2) / 2), 2) +
            Math.cos(lat1) * Math.cos(lat2) * Math.pow(Math.sin((lng1 - lng2) / 2), 2),
        ),
      );

    if (d < 1e-10) return [from, to];

    var points = [];
    var prevLng = from[1];
    for (var i = 0; i <= numPoints; i++) {
      var f = i / numPoints;
      var A = Math.sin((1 - f) * d) / Math.sin(d);
      var B = Math.sin(f * d) / Math.sin(d);
      var x = A * Math.cos(lat1) * Math.cos(lng1) + B * Math.cos(lat2) * Math.cos(lng2);
      var y = A * Math.cos(lat1) * Math.sin(lng1) + B * Math.cos(lat2) * Math.sin(lng2);
      var z = A * Math.sin(lat1) + B * Math.sin(lat2);
      var lat = (Math.atan2(z, Math.sqrt(x * x + y * y)) * 180) / Math.PI;
      var lng = (Math.atan2(y, x) * 180) / Math.PI;

      while (lng - prevLng > 180) lng -= 360;
      while (lng - prevLng < -180) lng += 360;
      prevLng = lng;

      points.push([lat, lng]);
    }
    return points;
  }

  function renderHops(hops) {
    routeLayer.clearLayers();
    var bounds = [];

    hops.forEach(function (hop) {
      if (
        hop.origin_lat == null ||
        hop.origin_lng == null ||
        hop.dest_lat == null ||
        hop.dest_lng == null
      )
        return;

      var from = [hop.origin_lat, hop.origin_lng];
      var to = [hop.dest_lat, hop.dest_lng];
      var color = colors[hop.travel_type] || '#6b7280';
      var emoji = emojis[hop.travel_type] || '';
      var points = arcPoints(from, to, 50);
      var dist = haversineKm(from[0], from[1], to[0], to[1]);
      var distStr = dist < 1 ? '<1' : Math.round(dist).toLocaleString();

      var startD = hop.start_date || '';
      var endD = hop.end_date || '';
      var dateStr = startD === endD ? startD : startD + ' \u2192 ' + endD;

      var popup =
        '<div class="hop-popup">' +
        '<div class="hop-popup-header">' +
        '<span class="hop-popup-emoji">' +
        emoji +
        '</span>' +
        '<span class="hop-popup-type">' +
        hop.travel_type.charAt(0).toUpperCase() +
        hop.travel_type.slice(1) +
        '</span>' +
        '</div>' +
        '<div class="hop-popup-route">' +
        '<div class="hop-popup-place">' +
        '<span class="hop-popup-label">\uD83D\uDFE2 From</span>' +
        '<strong>' +
        hop.origin_name +
        '</strong>' +
        (hop.origin_lat
          ? '<span class="hop-popup-coords">' +
            hop.origin_lat.toFixed(3) +
            ', ' +
            hop.origin_lng.toFixed(3) +
            '</span>'
          : '') +
        '</div>' +
        '<div class="hop-popup-arrow">\u2192</div>' +
        '<div class="hop-popup-place">' +
        '<span class="hop-popup-label">\uD83D\uDD34 To</span>' +
        '<strong>' +
        hop.dest_name +
        '</strong>' +
        (hop.dest_lat
          ? '<span class="hop-popup-coords">' +
            hop.dest_lat.toFixed(3) +
            ', ' +
            hop.dest_lng.toFixed(3) +
            '</span>'
          : '') +
        '</div>' +
        '</div>' +
        '<div class="hop-popup-details">' +
        '<div class="hop-popup-detail">\uD83D\uDCC5 ' +
        dateStr +
        '</div>' +
        '<div class="hop-popup-detail">\uD83D\uDCCF ' +
        distStr +
        ' km</div>' +
        '</div>' +
        '</div>';

      offsets.forEach(function (offset) {
        var shifted = points.map(function (p) {
          return [p[0], p[1] + offset];
        });
        L.polyline(shifted, {
          color: color,
          weight: 2.5,
          opacity: 0.75,
        })
          .bindPopup(popup, { maxWidth: 320, className: 'hop-popup-container' })
          .addTo(routeLayer);
      });

      bounds.push(from);
      bounds.push(to);
    });

    var cities = {};
    hops.forEach(function (hop) {
      if (hop.origin_lat != null && hop.origin_lng != null) {
        var oKey = hop.origin_name + '|' + hop.origin_lat + '|' + hop.origin_lng;
        if (!cities[oKey])
          cities[oKey] = {
            name: hop.origin_name,
            lat: hop.origin_lat,
            lng: hop.origin_lng,
            count: 0,
          };
        cities[oKey].count++;
      }
      if (hop.dest_lat != null && hop.dest_lng != null) {
        var dKey = hop.dest_name + '|' + hop.dest_lat + '|' + hop.dest_lng;
        if (!cities[dKey])
          cities[dKey] = { name: hop.dest_name, lat: hop.dest_lat, lng: hop.dest_lng, count: 0 };
        cities[dKey].count++;
      }
    });

    Object.keys(cities).forEach(function (key) {
      var c = cities[key];
      var r = Math.max(4, Math.min(8, 3 + Math.sqrt(c.count)));
      offsets.forEach(function (offset) {
        L.circleMarker([c.lat, c.lng + offset], {
          radius: r,
          color: isDark ? '#e5e5e5' : '#404040',
          weight: 1.5,
          fillColor: isDark ? '#f5f5f5' : '#171717',
          fillOpacity: 0.85,
        })
          .bindTooltip(c.name, { direction: 'top', offset: [0, -r], className: 'city-tooltip' })
          .addTo(routeLayer);
      });
    });

    document.getElementById('hop-count').textContent =
      hops.filter(function (h) {
        return (
          h.origin_lat != null && h.origin_lng != null && h.dest_lat != null && h.dest_lng != null
        );
      }).length + ' routes shown';

    if (bounds.length > 0) {
      map.fitBounds(bounds, { padding: [40, 40] });
    }
  }

  function applyFilters() {
    var typeVal = document.getElementById('filter-type').value;
    var yearVal = document.getElementById('filter-year').value;
    var filtered = allHops.filter(function (h) {
      if (typeVal !== 'all' && h.travel_type !== typeVal) return false;
      if (yearVal !== 'all' && !h.start_date.startsWith(yearVal)) return false;
      return true;
    });
    renderHops(filtered);
  }

  document.getElementById('filter-type').addEventListener('change', applyFilters);
  document.getElementById('filter-year').addEventListener('change', applyFilters);

  renderHops(allHops);
})();

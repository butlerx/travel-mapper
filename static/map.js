(function () {
  var allHops = window.allHops || [];
  var isDark = true;

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

  var darkTiles = L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}@2x.png', {
    attribution:
      '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
    maxZoom: 19,
    subdomains: 'abcd',
  });

  darkTiles.addTo(map);

  var colors = {
    air: '#38bdf8',
    rail: '#f59e0b',
    boat: '#f43f5e',
    transport: '#10b981',
  };
  var emojis = {
    air: '\u2708\uFE0F',
    rail: '\uD83D\uDE86',
    boat: '\uD83D\uDEA2',
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

  var routesLayer = L.layerGroup().addTo(map);
  var airportsLayer = L.layerGroup().addTo(map);
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

  function journeyCardHtml(hop) {
    var emoji = emojis[hop.travel_type] || '';
    var typeLabel = hop.travel_type.charAt(0).toUpperCase() + hop.travel_type.slice(1);
    var dist = '';
    if (
      hop.origin_lat != null &&
      hop.origin_lng != null &&
      hop.dest_lat != null &&
      hop.dest_lng != null
    ) {
      var km = haversineKm(hop.origin_lat, hop.origin_lng, hop.dest_lat, hop.dest_lng);
      dist = km < 1 ? '<1 km' : Math.round(km).toLocaleString() + ' km';
    }

    return (
      '<a href="/hop/' +
      hop.id +
      '" class="hop-card-link">' +
      '<div class="journey-card">' +
      '<div class="journey-route">' +
      '<span class="journey-origin">' +
      hop.origin_name +
      '</span>' +
      '<span class="journey-arrow">\u2192</span>' +
      '<span class="journey-dest">' +
      hop.dest_name +
      '</span>' +
      '</div>' +
      '<div class="journey-meta">' +
      '<span class="journey-badge badge-' +
      hop.travel_type +
      '">' +
      emoji +
      ' ' +
      typeLabel +
      '</span>' +
      '<span class="journey-date">' +
      hop.start_date +
      '</span>' +
      (dist ? '<span class="journey-distance">' + dist + '</span>' : '') +
      '</div>' +
      '</div>' +
      '</a>'
    );
  }

  function countdownText(dateStr) {
    var today = new Date();
    today.setHours(0, 0, 0, 0);
    var target = new Date(dateStr + 'T00:00:00');
    var diffMs = target - today;
    var days = Math.ceil(diffMs / 86400000);
    if (days === 0) return 'Today';
    if (days === 1) return 'Tomorrow';
    return 'In ' + days + ' days';
  }

  function renderJourneyCards(hops) {
    var sidebar = document.getElementById('journey-sidebar');
    if (!sidebar) return;

    if (hops.length === 0) {
      sidebar.innerHTML =
        '<h3 class="journey-sidebar-heading">Journeys</h3>' +
        '<div class="journey-empty">No journeys match the current filters.</div>';
      return;
    }

    var today = new Date().toISOString().slice(0, 10);
    var upcoming = [];
    var past = [];

    hops.forEach(function (hop) {
      if (hop.start_date >= today) {
        upcoming.push(hop);
      } else {
        past.push(hop);
      }
    });

    // Upcoming: soonest first
    upcoming.sort(function (a, b) {
      return a.start_date.localeCompare(b.start_date);
    });
    // Past: most recent first
    past.sort(function (a, b) {
      return b.start_date.localeCompare(a.start_date);
    });

    var html = '';

    if (upcoming.length > 0) {
      html +=
        '<h3 class="journey-sidebar-heading journey-sidebar-heading--upcoming">' +
        'Upcoming (' +
        upcoming.length +
        ')</h3>';
      upcoming.forEach(function (hop) {
        html +=
          '<a href="/hop/' +
          hop.id +
          '" class="hop-card-link">' +
          '<div class="journey-card journey-card--upcoming">' +
          '<div class="journey-route">' +
          '<span class="journey-origin">' +
          hop.origin_name +
          '</span>' +
          '<span class="journey-arrow">\u2192</span>' +
          '<span class="journey-dest">' +
          hop.dest_name +
          '</span>' +
          '</div>' +
          '<div class="journey-meta">' +
          '<span class="journey-badge badge-' +
          hop.travel_type +
          '">' +
          (emojis[hop.travel_type] || '') +
          ' ' +
          hop.travel_type.charAt(0).toUpperCase() +
          hop.travel_type.slice(1) +
          '</span>' +
          '<span class="journey-countdown">' +
          countdownText(hop.start_date) +
          '</span>' +
          '<span class="journey-date">' +
          hop.start_date +
          '</span>' +
          '</div>' +
          '</div>' +
          '</a>';
      });
    }

    if (past.length > 0) {
      html += '<h3 class="journey-sidebar-heading">Past Journeys (' + past.length + ')</h3>';
      past.forEach(function (hop) {
        html += journeyCardHtml(hop);
      });
    }

    if (upcoming.length === 0 && past.length === 0) {
      html += '<div class="journey-empty">No journeys match the current filters.</div>';
    }

    sidebar.innerHTML = html;
  }

  function renderHops(hops) {
    routesLayer.clearLayers();
    airportsLayer.clearLayers();
    var bounds = [];

    var routeFreq = {};
    hops.forEach(function (hop) {
      if (hop.origin_name && hop.dest_name) {
        var key1 = hop.origin_name + '→' + hop.dest_name;
        var key2 = hop.dest_name + '→' + hop.origin_name;
        var key = key1 < key2 ? key1 : key2;
        routeFreq[key] = (routeFreq[key] || 0) + 1;
      }
    });

    hops.forEach(function (hop) {
      if ((!hop.origin_lat && !hop.origin_lng) || (!hop.dest_lat && !hop.dest_lng)) return;

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

      var key1 = hop.origin_name + '→' + hop.dest_name;
      var key2 = hop.dest_name + '→' + hop.origin_name;
      var key = key1 < key2 ? key1 : key2;
      var freq = routeFreq[key] || 1;

      var weight = 2;
      var opacity = 0.6;
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
          weight: weight,
          opacity: opacity,
        })
          .bindPopup(popup, { maxWidth: 320, className: 'hop-popup-container' })
          .addTo(routesLayer);
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
            routes: {},
          };
        cities[oKey].count++;
        cities[oKey].routes[hop.dest_name] = (cities[oKey].routes[hop.dest_name] || 0) + 1;
      }
      if (hop.dest_lat != null && hop.dest_lng != null) {
        var dKey = hop.dest_name + '|' + hop.dest_lat + '|' + hop.dest_lng;
        if (!cities[dKey])
          cities[dKey] = {
            name: hop.dest_name,
            lat: hop.dest_lat,
            lng: hop.dest_lng,
            count: 0,
            routes: {},
          };
        cities[dKey].count++;
        cities[dKey].routes[hop.origin_name] = (cities[dKey].routes[hop.origin_name] || 0) + 1;
      }
    });

    Object.keys(cities).forEach(function (key) {
      var c = cities[key];
      var r = Math.max(4, Math.min(8, 3 + Math.sqrt(c.count)));
      var markerOpts = {
        radius: r,
        color: '#1e3a5f',
        weight: 1.5,
        fillColor: '#38bdf8',
        fillOpacity: 0.85,
      };
      var sortedRoutes = Object.keys(c.routes)
        .map(function (dest) {
          return { name: dest, count: c.routes[dest] };
        })
        .sort(function (a, b) {
          return b.count - a.count;
        });
      var routeListHtml = sortedRoutes
        .slice(0, 5)
        .map(function (rt) {
          return (
            '<div class="airport-popup-route">' +
            '<span class="airport-popup-dest">' +
            rt.name +
            '</span>' +
            '<span class="airport-popup-freq">' +
            rt.count +
            '\u00d7</span>' +
            '</div>'
          );
        })
        .join('');
      if (sortedRoutes.length > 5) {
        routeListHtml +=
          '<div class="airport-popup-more">+' +
          (sortedRoutes.length - 5) +
          ' more destinations</div>';
      }
      var popupHtml =
        '<div class="airport-popup">' +
        '<div class="airport-popup-header">' +
        '<strong>' +
        c.name +
        '</strong>' +
        '</div>' +
        '<div class="airport-popup-stats">' +
        '<span class="airport-popup-visits">' +
        c.count +
        ' visit' +
        (c.count !== 1 ? 's' : '') +
        '</span>' +
        '<span class="airport-popup-connections">' +
        sortedRoutes.length +
        ' connection' +
        (sortedRoutes.length !== 1 ? 's' : '') +
        '</span>' +
        '</div>' +
        '<div class="airport-popup-routes">' +
        routeListHtml +
        '</div>' +
        '</div>';

      offsets.forEach(function (offset) {
        var marker = L.circleMarker([c.lat, c.lng + offset], markerOpts)
          .bindPopup(popupHtml, { maxWidth: 280, className: 'airport-popup-container' })
          .on('mouseover', function (e) {
            this.openPopup();
          })
          .on('mouseout', function (e) {
            if (!this._popupHandlingClick) {
              this.closePopup();
            }
          })
          .on('click', function () {
            this._popupHandlingClick = true;
            this.openPopup();
          })
          .on('popupclose', function () {
            this._popupHandlingClick = false;
          });
        marker.addTo(airportsLayer);
      });
    });

    document.getElementById('hop-count').textContent =
      hops.filter(function (h) {
        return (
          h.origin_lat != null && h.origin_lng != null && h.dest_lat != null && h.dest_lng != null
        );
      }).length + ' journeys';

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
    renderJourneyCards(filtered);
  }

  document.getElementById('filter-type').addEventListener('change', applyFilters);
  document.getElementById('filter-year').addEventListener('change', applyFilters);

  var toggleRoutes = document.getElementById('toggle-routes');
  var toggleAirports = document.getElementById('toggle-airports');
  if (toggleRoutes) {
    toggleRoutes.addEventListener('change', function () {
      if (this.checked) {
        map.addLayer(routesLayer);
      } else {
        map.removeLayer(routesLayer);
      }
    });
  }
  if (toggleAirports) {
    toggleAirports.addEventListener('change', function () {
      if (this.checked) {
        map.addLayer(airportsLayer);
      } else {
        map.removeLayer(airportsLayer);
      }
    });
  }

  renderHops(allHops);
  renderJourneyCards(allHops);
})();

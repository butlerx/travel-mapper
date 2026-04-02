// @ts-check

/** @type {HTMLElement | null} */
const el = document.getElementById('trip-map');
if (el && typeof L !== 'undefined') {
  /** @type {Array<{oLat: number, oLng: number, dLat: number, dLng: number}>} */
  const legs = JSON.parse(el.dataset.legs || '[]');
  if (legs.length > 0) {
    const map = L.map('trip-map', { scrollWheelZoom: false });
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '&copy; OpenStreetMap contributors',
      maxZoom: 18,
    }).addTo(map);

    /** @type {Array<[number, number]>} */
    const allPoints = [];
    /** @type {Set<string>} */
    const markerKeys = new Set();

    for (const leg of legs) {
      L.polyline(
        [
          [leg.oLat, leg.oLng],
          [leg.dLat, leg.dLng],
        ],
        { color: '#4a90d9', weight: 3, dashArray: '8 4' },
      ).addTo(map);

      const oKey = `${leg.oLat},${leg.oLng}`;
      const dKey = `${leg.dLat},${leg.dLng}`;
      if (!markerKeys.has(oKey)) {
        L.marker([leg.oLat, leg.oLng]).addTo(map);
        markerKeys.add(oKey);
      }
      if (!markerKeys.has(dKey)) {
        L.marker([leg.dLat, leg.dLng]).addTo(map);
        markerKeys.add(dKey);
      }

      allPoints.push([leg.oLat, leg.oLng]);
      allPoints.push([leg.dLat, leg.dLng]);
    }

    map.fitBounds(allPoints, { padding: [40, 40] });
  }
}

// @ts-check

/** @type {HTMLElement | null} */
const el = document.getElementById('journey-map');
if (el && typeof L !== 'undefined') {
  /** @type {number} */
  const oLat = parseFloat(el.dataset.originLat);
  /** @type {number} */
  const oLng = parseFloat(el.dataset.originLng);
  /** @type {number} */
  const dLat = parseFloat(el.dataset.destLat);
  /** @type {number} */
  const dLng = parseFloat(el.dataset.destLng);
  if (!isNaN(oLat) && !isNaN(dLat)) {
    const map = L.map('journey-map', { scrollWheelZoom: false });
    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '&copy; OpenStreetMap contributors',
      maxZoom: 18,
    }).addTo(map);
    L.marker([oLat, oLng]).addTo(map);
    L.marker([dLat, dLng]).addTo(map);
    L.polyline(
      [
        [oLat, oLng],
        [dLat, dLng],
      ],
      {
        color: '#4a90d9',
        weight: 3,
        dashArray: '8 4',
      },
    ).addTo(map);
    map.fitBounds(
      [
        [oLat, oLng],
        [dLat, dLng],
      ],
      { padding: [40, 40] },
    );
  }
}

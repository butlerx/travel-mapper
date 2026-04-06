/** @type {HTMLElement | null} */
const page = /** @type {HTMLElement | null} */ (document.querySelector('[data-trip-id]'));
if (page) {
  const tripId = page.dataset.tripId;

  const deleteBtn = document.querySelector('[data-delete-trip]');
  if (deleteBtn) {
    deleteBtn.addEventListener('click', () => {
      if (confirm('Delete this trip? Journeys will remain but be unassigned.')) {
        fetch(`/trips/${tripId}`, { method: 'DELETE' }).then((r) => {
          if (r.ok) window.location.href = '/trips';
          else window.location.reload();
        });
      }
    });
  }

  document.querySelectorAll('[data-remove-journey]').forEach((btn) => {
    btn.addEventListener('click', () => {
      const journeyId = /** @type {HTMLElement} */ (btn).dataset.removeJourney;
      if (confirm('Remove this journey from the trip?')) {
        fetch(`/trips/${tripId}/journeys/${journeyId}`, {
          method: 'DELETE',
        }).then((r) => {
          if (r.ok) window.location.reload();
        });
      }
    });
  });
}

// @ts-check

/**
 * Live status polling for the Upcoming page. Cards for journeys departing today
 * carry `data-live="1"`; for each we poll the existing enrichment endpoint and
 * refresh the status badge in place, so the page tracks delays/gate changes
 * without a manual reload.
 */

const liveCards = document.querySelectorAll('[data-live="1"]');

/**
 * @param {string} status
 * @param {number | null | undefined} delay
 * @returns {string}
 */
function statusLabel(status, delay) {
  if (delay != null && delay > 0) return `${status} (+${delay}m)`;
  if (delay != null && delay < 0) return `${status} (${delay}m)`;
  return status;
}

/** @param {Element} card */
async function refreshCard(card) {
  const id = card.getAttribute('data-journey-id');
  const target = card.querySelector('[data-live-status]');
  if (!id || !target) return;
  try {
    const res = await fetch(`/journeys/${id}/enrichments`, {
      headers: { Accept: 'application/json' },
    });
    if (!res.ok) return;
    const items = await res.json();
    const withStatus = items.filter((e) => e && e.status);
    if (withStatus.length === 0) return;
    withStatus.sort((a, b) => String(b.fetched_at || '').localeCompare(String(a.fetched_at || '')));
    const latest = withStatus[0];

    const badge = document.createElement('span');
    badge.className = `status-badge status-${String(latest.status).toLowerCase().replace(/ /g, '-')}`;
    badge.textContent = statusLabel(latest.status, latest.delay_minutes);
    target.replaceChildren(badge);
  } catch {
    // Network/parse failures are non-fatal — keep the last-rendered status.
  }
}

function refreshAll() {
  liveCards.forEach((card) => {
    void refreshCard(card);
  });
}

if (liveCards.length > 0) {
  refreshAll();
  setInterval(refreshAll, 60000);
}

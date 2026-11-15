/**
 * Live status polling for shared journey pages.
 */

interface Enrichment {
  status?: string;
  delay_minutes?: number | null;
  fetched_at?: string;
}

const pageRoot = document.querySelector('[data-enrichment-url]');

function statusLabel(status: string, delay: number | null | undefined): string {
  if (delay != null && delay > 0) return `${status} (+${delay}m)`;
  if (delay != null && delay < 0) return `${status} (${delay}m)`;
  return status;
}

async function refreshStatus(): Promise<void> {
  if (!pageRoot) return;
  const enrichmentUrl = pageRoot.getAttribute('data-enrichment-url');
  const target = pageRoot.querySelector('[data-shared-status]');
  if (!enrichmentUrl || !target) return;

  try {
    const response = await fetch(enrichmentUrl, {
      headers: { Accept: 'application/json' },
    });
    if (!response.ok) return;
    const items: Enrichment[] = await response.json();
    const withStatus = items.filter((item) => item && item.status);
    if (withStatus.length === 0) return;
    withStatus.sort((a, b) => String(b.fetched_at || '').localeCompare(String(a.fetched_at || '')));
    const latest = withStatus[0];
    const badge = document.createElement('span');
    badge.className = `status-badge status-${String(latest.status).toLowerCase().replace(/ /g, '-')}`;
    badge.textContent = statusLabel(latest.status!, latest.delay_minutes);
    target.replaceChildren(badge);
  } catch {
    // Keep the currently rendered value on errors.
  }
}

if (pageRoot) {
  void refreshStatus();
  setInterval(() => {
    void refreshStatus();
  }, 60000);
}

export {};

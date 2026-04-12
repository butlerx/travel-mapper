// Service Worker for Travel Export PWA
'use strict';

const CACHE_VERSION = 'v2';
const STATIC_CACHE = `travel-static-${CACHE_VERSION}`;

const PRECACHE_URLS = [
  '/static/style.css',
  '/static/map.js',
  '/static/stats-map.js',
  '/static/nav.js',
  '/static/add-journey.js',
  '/static/journey-map.js',
  '/static/icons/logo.svg',
  '/static/icons/plane.svg',
  '/static/icons/train.svg',
  '/static/icons/boat.svg',
  '/static/icons/transport.svg',
  '/manifest.json',
];

// Install: precache static assets
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(STATIC_CACHE)
      .then((cache) => cache.addAll(PRECACHE_URLS))
      .then(() => self.skipWaiting()),
  );
});

// Activate: remove old caches
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(keys.filter((key) => key !== STATIC_CACHE).map((key) => caches.delete(key))),
      )
      .then(() => self.clients.claim()),
  );
});

// Fetch: cache-first for static assets, network-first for everything else
self.addEventListener('fetch', (event) => {
  const { request } = event;

  // Only handle GET requests
  if (request.method !== 'GET') return;

  const url = new URL(request.url);

  // Cache-first for static assets and manifest
  if (url.pathname.startsWith('/static/') || url.pathname === '/manifest.json') {
    event.respondWith(
      caches.match(request).then(
        (cached) =>
          cached ||
          fetch(request).then((response) => {
            if (response.ok) {
              const clone = response.clone();
              caches.open(STATIC_CACHE).then((cache) => cache.put(request, clone));
            }
            return response;
          }),
      ),
    );
    return;
  }

  // Network-first for pages and API
  event.respondWith(fetch(request).catch(() => caches.match(request)));
});

// Push: show notification from server
self.addEventListener('push', (event) => {
  const data = event.data ? event.data.json() : {};
  const title = data.title || 'Travel Mapper';
  const options = {
    body: data.body || '',
    icon: '/static/logo.svg',
    badge: '/static/logo.svg',
    data: { url: data.url || '/' },
  };
  event.waitUntil(self.registration.showNotification(title, options));
});

// Notification click: open or focus the target page
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  const url = event.notification.data?.url || '/';
  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true }).then((windowClients) => {
      for (const client of windowClients) {
        if (client.url.includes(url) && 'focus' in client) {
          return client.focus();
        }
      }
      return clients.openWindow(url);
    }),
  );
});

// Subscription change: re-subscribe and update server
self.addEventListener('pushsubscriptionchange', (event) => {
  event.waitUntil(
    self.registration.pushManager.subscribe(event.oldSubscription.options).then((newSub) =>
      fetch('/auth/push-subscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newSub.toJSON()),
      }),
    ),
  );
});

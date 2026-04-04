(async () => {
  const button = document.getElementById('push-toggle');
  const status = document.getElementById('push-status');
  const config = document.getElementById('push-config');
  if (!button || !status || !config) {
    return;
  }

  const supported = 'PushManager' in window && 'serviceWorker' in navigator;
  const vapidKey = config.dataset.vapidKey || '';
  let registration = null;
  let subscription = null;

  const setSubscribed = (isSubscribed) => {
    button.dataset.subscribed = isSubscribed ? 'true' : 'false';
    button.textContent = isSubscribed ? 'Disable Push Notifications' : 'Enable Push Notifications';
    button.disabled = false;
    status.textContent = isSubscribed
      ? 'Push notifications are enabled.'
      : 'Push notifications are disabled.';
  };

  const urlBase64ToUint8Array = (base64String) => {
    const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
    const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
    const rawData = atob(base64);
    return Uint8Array.from([...rawData].map((char) => char.charCodeAt(0)));
  };

  if (!supported) {
    status.textContent = 'Push notifications are not supported in this browser.';
    button.disabled = true;
    return;
  }

  try {
    registration = await navigator.serviceWorker.getRegistration();
    if (!registration) {
      registration = await navigator.serviceWorker.register('/sw.js');
    }
    subscription = await registration.pushManager.getSubscription();
    setSubscribed(Boolean(subscription));
  } catch (error) {
    console.warn('Failed to load push status', error);
    status.textContent = 'Failed to check push notification status.';
    button.disabled = true;
    return;
  }

  button.addEventListener('click', async () => {
    button.disabled = true;
    try {
      if (button.dataset.subscribed === 'true') {
        if (subscription) {
          await fetch('/auth/push-subscribe', {
            method: 'DELETE',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ endpoint: subscription.endpoint }),
          });
          await subscription.unsubscribe();
          subscription = null;
        }
        setSubscribed(false);
        return;
      }

      subscription = await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: urlBase64ToUint8Array(vapidKey),
      });

      await fetch('/auth/push-subscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(subscription.toJSON()),
      });

      setSubscribed(true);
    } catch (error) {
      console.warn('Failed to update push subscription', error);
      status.textContent = 'Failed to update push notification setting.';
      button.disabled = false;
    }
  });
})();

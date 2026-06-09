document.querySelectorAll('[data-copy-trigger]').forEach((btn) => {
  btn.addEventListener('click', () => {
    const container = btn.closest('.new-token-value');
    const code = container?.querySelector('[data-copy-value]');
    const text = code?.getAttribute('data-copy-value');
    if (!text) return;
    navigator.clipboard.writeText(text).then(() => {
      btn.textContent = 'Copied!';
      setTimeout(() => {
        btn.textContent = 'Copy';
      }, 2000);
    });
  });
});

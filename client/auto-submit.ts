document.querySelectorAll('[data-auto-submit]').forEach((el) => {
  el.addEventListener('change', () => {
    const form = el.closest('form');
    if (form) form.submit();
  });
});

document.querySelectorAll('[data-auto-submit]').forEach(function (el) {
  el.addEventListener('change', function () {
    /** @type {HTMLFormElement | null} */
    const form = el.closest('form');
    if (form) form.submit();
  });
});

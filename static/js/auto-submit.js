document.querySelectorAll('[data-auto-submit]').forEach(function (el) {
  el.addEventListener('change', function () {
    el.form.submit();
  });
});

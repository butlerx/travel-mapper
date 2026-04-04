document.querySelectorAll('[data-copy-trigger]').forEach(function (btn) {
  btn.addEventListener('click', function () {
    var code = btn.closest('.new-token-value').querySelector('[data-copy-value]');
    var text = code.getAttribute('data-copy-value');
    navigator.clipboard.writeText(text).then(function () {
      btn.textContent = 'Copied!';
      setTimeout(function () {
        btn.textContent = 'Copy';
      }, 2000);
    });
  });
});

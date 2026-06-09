const form = document.getElementById('edit-form');
const backdrop = document.getElementById('edit-backdrop');

if (form && backdrop) {
  const open = () => {
    form.classList.add('open');
    backdrop.classList.add('open');
  };

  const close = () => {
    form.classList.remove('open');
    backdrop.classList.remove('open');
  };

  document.querySelectorAll('[data-edit-open]').forEach((el) => {
    el.addEventListener('click', open);
  });

  document.querySelectorAll('[data-edit-close]').forEach((el) => {
    el.addEventListener('click', close);
  });

  backdrop.addEventListener('click', close);
}

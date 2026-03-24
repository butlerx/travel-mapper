// @ts-check

/** @type {HTMLSelectElement} */
const sel = /** @type {HTMLSelectElement} */ (document.getElementById('travel_type'));
const originInput = /** @type {HTMLInputElement} */ (document.getElementById('origin'));
/** @type {HTMLInputElement} */
const dest = /** @type {HTMLInputElement} */ (document.getElementById('destination'));
/** @type {HTMLElement} */
const originLabel = /** @type {HTMLElement} */ (document.getElementById('origin-label'));
/** @type {HTMLElement} */
const destLabel = /** @type {HTMLElement} */ (document.getElementById('destination-label'));
/** @type {NodeListOf<HTMLElement>} */
const sections = document.querySelectorAll('.type-fields');

/** @returns {void} */
function update() {
  const t = sel.value;
  sections.forEach((s) => {
    s.style.display = 'none';
  });
  const active = document.getElementById(`fields-${t}`);
  if (active) active.style.display = '';

  if (t === 'air') {
    originLabel.textContent = 'Origin (IATA code)';
    destLabel.textContent = 'Destination (IATA code)';
    originInput.placeholder = 'LHR';
    originInput.maxLength = 4;
    originInput.style.textTransform = 'uppercase';
    dest.placeholder = 'JFK';
    dest.maxLength = 4;
    dest.style.textTransform = 'uppercase';
  } else {
    originLabel.textContent = 'Origin';
    destLabel.textContent = 'Destination';
    originInput.placeholder = 'Paris Gare du Nord';
    originInput.removeAttribute('maxlength');
    originInput.style.textTransform = '';
    dest.placeholder = 'London St Pancras';
    dest.removeAttribute('maxlength');
    dest.style.textTransform = '';
  }
}

sel.addEventListener('change', update);
update();

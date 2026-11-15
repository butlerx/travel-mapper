const sel = document.getElementById('travel_type') as HTMLSelectElement;
const originInput = document.getElementById('origin') as HTMLInputElement;
const dest = document.getElementById('destination') as HTMLInputElement;
const originLabel = document.getElementById('origin-label') as HTMLElement;
const destLabel = document.getElementById('destination-label') as HTMLElement;
const sections = document.querySelectorAll<HTMLElement>('.type-fields');

function update(): void {
  const t = sel.value;
  sections.forEach((s) => {
    s.classList.toggle('is-active', s.id === `fields-${t}`);
  });

  if (t === 'air') {
    originLabel.textContent = 'Origin (IATA code)';
    destLabel.textContent = 'Destination (IATA code)';
    originInput.placeholder = 'LHR';
    originInput.maxLength = 4;
    originInput.classList.add('uppercase-input');
    dest.placeholder = 'JFK';
    dest.maxLength = 4;
    dest.classList.add('uppercase-input');
  } else {
    originLabel.textContent = 'Origin';
    destLabel.textContent = 'Destination';
    originInput.placeholder = 'Paris Gare du Nord';
    originInput.removeAttribute('maxlength');
    originInput.classList.remove('uppercase-input');
    dest.placeholder = 'London St Pancras';
    dest.removeAttribute('maxlength');
    dest.classList.remove('uppercase-input');
  }
}

sel.addEventListener('change', update);
update();

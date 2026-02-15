// Contextual sidebar loader (navigation.json).
(function contextualSidebar() {
  const sidebar = document.querySelector('#mdbook-sidebar .sidebar-scrollbox');
  if (!sidebar) {
    return;
  }

  const navUrl = new URL('navigation.json', window.location.href);
  console.log('[nav] navigation.json url', navUrl.href);

  function normalizeItems(data) {
    if (Array.isArray(data)) {
      return data;
    }
    if (data && Array.isArray(data.items)) {
      return data.items;
    }
    return null;
  }

  function buildList(items, isRoot) {
    const list = document.createElement('ol');
    list.className = isRoot ? 'chapter' : 'section';

    items.forEach(item => {
      if (!item || typeof item !== 'object') {
        return;
      }

      if (item.type === 'part' || item.part === true) {
        const part = document.createElement('li');
        part.className = 'part-title';
        part.textContent = item.title || item.text || '';
        list.appendChild(part);
        return;
      }

      const li = document.createElement('li');
      li.className = 'chapter-item expanded';

      const wrapper = document.createElement('span');
      wrapper.className = 'chapter-link-wrapper';
      const title = item.title || item.text || '';
      const href = item.href || item.link || item.url;
      if (href) {
        const link = document.createElement('a');
        link.setAttribute('href', href);
        link.textContent = title;
        wrapper.appendChild(link);
      } else {
        const span = document.createElement('span');
        span.textContent = title;
        wrapper.appendChild(span);
      }
      li.appendChild(wrapper);

      if (Array.isArray(item.children) && item.children.length > 0) {
        li.appendChild(buildList(item.children, false));
      }
      list.appendChild(li);
    });

    return list;
  }

  function markActive(container) {
    const current = window.location.href.toString().split('#')[0].split('?')[0];
    const links = Array.from(container.querySelectorAll('a'));
    links.forEach(link => {
      const href = link.getAttribute('href');
      if (!href || href.startsWith('#') || /^(?:[a-z+]+:)?\/\//.test(href)) {
        return;
      }
      const resolved = new URL(href, window.location.href).href;
      link.href = resolved;
      if (resolved === current) {
        link.classList.add('active');
        let parent = link.parentElement;
        while (parent) {
          if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
            parent.classList.add('expanded');
          }
          parent = parent.parentElement;
        }
      }
    });
  }

  fetch(navUrl, { cache: 'no-store' })
    .then(response => {
      if (!response.ok) {
        console.log('[nav] navigation.json missing', response.status);
        return null;
      }
      return response.json();
    })
    .then(data => {
      if (!data) {
        return;
      }
      console.log('[nav] navigation.json loaded', data);
      const items = normalizeItems(data);
      if (!items) {
        console.log('[nav] navigation.json invalid format');
        return;
      }
      const list = buildList(items, true);
      sidebar.innerHTML = '';
      sidebar.appendChild(list);
      markActive(sidebar);
    })
    .catch(error => {
      console.log('[nav] navigation.json error', error);
    });
})();

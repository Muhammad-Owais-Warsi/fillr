let activeInput = null;
let popup = null;
let isActive = false;
let query = "";
let selectedIndex = 0;

let SNIPPET_DATA = {};
let items = [];

function refreshData() {
  chrome.storage.local.get("myAppData", (result) => {
    SNIPPET_DATA = result.myAppData || {
      github: "https://github.com/user",
      email: "user@example.com",
    };
    items = Object.keys(SNIPPET_DATA);
  });
}

refreshData();

chrome.runtime.onMessage.addListener((msg) => {
  if (msg.action === "REFRESH_DATA") refreshData();
});

document.addEventListener("focusin", (e) => {
  if (
    e.target.tagName === "INPUT" ||
    e.target.tagName === "TEXTAREA" ||
    e.target.isContentEditable
  ) {
    activeInput = e.target;
    activeInput.setAttribute("autocomplete", "off");
  }
});

document.addEventListener("keydown", (e) => {
  if (!activeInput) return;

  if ((e.key === ";" || e.code === "Semicolon") && !isActive && (e.ctrlKey || e.metaKey)) {
    isActive = true;
    query = "";
    selectedIndex = 0;
    e.preventDefault();
    showPopup();
    return;
  }

  if (!isActive) return;

  if (e.key === "ArrowDown") {
    e.preventDefault();
    selectedIndex = (selectedIndex + 1) % items.length;
    renderPopup();
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    selectedIndex = (selectedIndex - 1 + items.length) % items.length;
    renderPopup();
  } else if (e.key === "Enter" || e.key === "Tab") {
    if (items.length > 0) {
      e.preventDefault();
      insertValue(items[selectedIndex]);
      closePopup();
    }
  } else if (e.key === "Escape") {
    closePopup();
  } else if (e.key === "Backspace") {
    if (query.length === 0) {
      closePopup();
    } else {
      query = query.slice(0, -1);
      filterItems();
      renderPopup();
    }
    e.preventDefault();
  } else if (e.key.length === 1 && e.key !== ";" && e.code !== "Semicolon") {
    e.preventDefault();
    query += e.key.toLowerCase();
    filterItems();
    if (items.length === 0) {
      closePopup();
      activeInput.focus();
      document.execCommand('insertText', false, e.key);
    } else {
      renderPopup();
    }
  } else if (!e.ctrlKey && !e.metaKey && !e.altKey) {
    closePopup();
  }
});

document.addEventListener(
  "mousedown",
  (e) => {
    if (!isActive || !popup) return;
    if (!popup.contains(e.target) && e.target !== activeInput) {
      closePopup();
    }
  },
  true,
);

function filterItems() {
  items = Object.keys(SNIPPET_DATA).filter((k) =>
    k.toLowerCase().includes(query),
  );
  selectedIndex = 0;
}

function showPopup() {
  if (popup) popup.remove();

  popup = document.createElement("div");
  popup.className = "equals-popup";

  // Render off-screen first so we can measure its real height
  popup.style.cssText =
    "position:fixed; z-index:999999999; visibility:hidden; top:-9999px; left:-9999px;";
  document.body.appendChild(popup);

  filterItems();
  renderPopup(false); // render content without repositioning yet

  // Now that content is rendered, measure and position correctly
  positionPopup();
  popup.style.visibility = "visible";
}

function positionPopup() {
  if (!popup || !activeInput) return;

  const rect = activeInput.getBoundingClientRect();
  const GAP = 6;
  const popupHeight = popup.offsetHeight; // accurate now — content already rendered
  const popupWidth = popup.offsetWidth;

  const spaceBelow = window.innerHeight - rect.bottom;
  const spaceAbove = rect.top;
  const showAbove = spaceBelow < popupHeight + GAP && spaceAbove > spaceBelow;

  // Vertical
  popup.style.top = showAbove
    ? `${rect.top - popupHeight - GAP}px`
    : `${rect.bottom + GAP}px`;

  // Horizontal — keep within viewport
  let left = rect.left;
  if (left + popupWidth > window.innerWidth - 8) {
    left = window.innerWidth - popupWidth - 8;
  }
  if (left < 8) left = 8;
  popup.style.left = `${left}px`;
}

function renderPopup(reposition = true) {
  if (!popup) return;

  if (items.length === 0) {
    popup.innerHTML = `

      <div class="item no-match">no matches</div>
    `;
  } else {
    popup.innerHTML = `
      ${items
        .map(
          (item, i) => `
        <div class="item ${i === selectedIndex ? "active" : ""}" data-key="${item}">
          <span class="item-key">${item}</span>
        </div>
      `,
        )
        .join("")}
    `;

    popup.querySelectorAll(".item").forEach((el) => {
      el.addEventListener("mousedown", (e) => {
        e.preventDefault();
        insertValue(el.dataset.key);
        closePopup();
      });
    });
  }

  // Re-position after each render since height may change as results filter
  if (reposition) positionPopup();
}

function insertValue(key) {
  const val = SNIPPET_DATA[key];
  if (activeInput.tagName === "INPUT" || activeInput.tagName === "TEXTAREA") {
    const start = activeInput.selectionStart;
    const end = activeInput.selectionEnd;
    activeInput.value =
      activeInput.value.slice(0, start) + val + activeInput.value.slice(end);
    activeInput.selectionStart = activeInput.selectionEnd = start + val.length;
  } else {
    activeInput.focus();
    document.execCommand("insertText", false, val);
  }
  activeInput.dispatchEvent(new Event("input", { bubbles: true }));
}

function closePopup() {
  isActive = false;
  if (popup) {
    popup.remove();
    popup = null;
  }
}

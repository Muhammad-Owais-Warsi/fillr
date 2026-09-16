const keyInput = document.getElementById("key");
const valInput = document.getElementById("val");
const saveBtn = document.getElementById("saveBtn");
const snippetList = document.getElementById("snippetList");
const snippetCount = document.getElementById("snippet-count");
const exportBtn = document.getElementById("exportBtn");
const importBtn = document.getElementById("importBtn");
const importFile = document.getElementById("importFile");

document.addEventListener("DOMContentLoaded", loadSnippets);

function loadSnippets() {
  chrome.storage.local.get("myAppData", (res) => {
    const data = res.myAppData || {};
    renderList(data);
  });
}

function renderList(data) {
  snippetList.innerHTML = "";
  const keys = Object.keys(data);

  snippetCount.innerText = keys.length.toString().padStart(2, "0");

  if (keys.length === 0) {
    snippetList.innerHTML = '<div class="empty-state">NO SNIPPETS SAVED</div>';
    return;
  }

  keys.forEach((key) => {
    const item = document.createElement("div");
    item.className = "snippet-item";
    item.innerHTML = `
      <div class="snippet-key">${key}</div>
      <div class="snippet-val">${data[key]}</div>
      <div class="actions">
        <button class="btn-edit" data-key="${key}">EDIT</button>
        <button class="btn-del" data-key="${key}">DEL</button>
      </div>
    `;
    snippetList.appendChild(item);
  });

  document.querySelectorAll(".btn-del").forEach((btn) => {
    btn.onclick = () => deleteSnippet(btn.dataset.key);
  });

  document.querySelectorAll(".btn-edit").forEach((btn) => {
    btn.onclick = () => editSnippet(btn.dataset.key, data[btn.dataset.key]);
  });
}

saveBtn.onclick = () => {
  const key = keyInput.value.trim();
  const val = valInput.value.trim();

  if (!key || !val) return;

  chrome.storage.local.get("myAppData", (res) => {
    const data = res.myAppData || {};
    data[key] = val;

    chrome.storage.local.set({ myAppData: data }, () => {
      keyInput.value = "";
      valInput.value = "";
      saveBtn.innerText = "ADD";
      loadSnippets();
      notifyAllTabs();
    });
  });
};

function deleteSnippet(key) {
  chrome.storage.local.get("myAppData", (res) => {
    const data = res.myAppData || {};
    delete data[key];
    chrome.storage.local.set({ myAppData: data }, () => {
      loadSnippets();
      notifyAllTabs();
    });
  });
}

// EDIT
function editSnippet(key, val) {
  keyInput.value = key;
  valInput.value = val;
  saveBtn.innerText = "SAVE";
  keyInput.focus();
}

function notifyAllTabs() {
  chrome.tabs.query({}, (tabs) => {
    tabs.forEach((tab) => {
      if (
        tab.url &&
        (tab.url.startsWith("http") || tab.url.startsWith("https"))
      ) {
        chrome.tabs
          .sendMessage(tab.id, { action: "REFRESH_DATA" })
          .catch(() => {});
      }
    });
  });
}

exportBtn.onclick = () => {
  chrome.storage.local.get("myAppData", (res) => {
    const data = res.myAppData || {};
    const blob = new Blob([JSON.stringify(data, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "pop-snippets.json";
    a.click();
    URL.revokeObjectURL(url);
  });
};

importBtn.onclick = () => importFile.click();

importFile.onchange = () => {
  const file = importFile.files[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    try {
      const parsed = JSON.parse(reader.result);
      if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
        throw new Error("invalid format");
      }
      const clean = {};
      Object.entries(parsed).forEach(([k, v]) => {
        if (typeof k === "string" && k.trim() && typeof v === "string") {
          clean[k.trim()] = v;
        }
      });
      chrome.storage.local.get("myAppData", (res) => {
        const data = { ...(res.myAppData || {}), ...clean };
        chrome.storage.local.set({ myAppData: data }, () => {
          importFile.value = "";
          loadSnippets();
          notifyAllTabs();
        });
      });
    } catch {
      importFile.value = "";
    }
  };
  reader.readAsText(file);
};

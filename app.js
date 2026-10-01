'use strict';

// --- state ----------------------------------------------------------------

const STORAGE_KEY = 'td:tasks:v1';
const UNDO_LIMIT = 5;
const SAVE_DEBOUNCE_MS = 150;
const DELETE_ALL_CONFIRM_MS = 3000;

let tasks = [];
let undoStack = [];
let hoveredId = null;
let selectedId = null;
let expandedId = null;
let editingTitleId = null;
let draggingId = null;
let saveTimer = null;
let deleteAllArmed = false;
let deleteAllTimer = null;

// --- dom refs -------------------------------------------------------------

const listEl = document.getElementById('task-list');
const formEl = document.getElementById('new-task-form');
const inputTitle = document.getElementById('new-title');
const inputDesc = document.getElementById('new-description');
const btnAddNote = document.getElementById('add-note-btn');
const deleteAllBtn = document.getElementById('delete-all-btn');

// --- helpers --------------------------------------------------------------

function uid() {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) return crypto.randomUUID();
  return 'id-' + Date.now().toString(36) + '-' + Math.random().toString(36).slice(2, 10);
}

function isTypingTarget(target) {
  if (!target) return false;
  const tag = target.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  if (target.isContentEditable) return true;
  return false;
}

function clampRows(text) {
  const lines = text.split('\n').length;
  return Math.min(8, Math.max(2, lines + 1));
}

function getActiveId() {
  return hoveredId ?? selectedId ?? null;
}

// --- persistence ----------------------------------------------------------

function load() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return;
    tasks = parsed
      .filter(t => t && typeof t.id === 'string' && typeof t.title === 'string')
      .map(t => ({
        id: t.id,
        title: t.title,
        description: typeof t.description === 'string' ? t.description : '',
        completed: !!t.completed,
      }));
  } catch (err) {
    console.warn('Failed to load tasks from localStorage:', err);
    tasks = [];
  }
}

function scheduleSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(saveNow, SAVE_DEBOUNCE_MS);
}

function saveNow() {
  saveTimer = null;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(tasks));
  } catch (err) {
    console.warn('Failed to save tasks to localStorage:', err);
  }
}

// --- operations -----------------------------------------------------------

function addTask(title, description) {
  tasks.push({
    id: uid(),
    title: title,
    description: description,
    completed: false,
  });
  render();
  scheduleSave();
}

function deleteTask(id) {
  const idx = tasks.findIndex(t => t.id === id);
  if (idx < 0) return;
  const [removed] = tasks.splice(idx, 1);
  removed.__originalIndex = idx;
  undoStack.push(removed);
  if (undoStack.length > UNDO_LIMIT) undoStack.shift();
  if (hoveredId === id) hoveredId = null;
  if (selectedId === id) selectedId = null;
  if (expandedId === id) expandedId = null;
  if (editingTitleId === id) editingTitleId = null;
  render();
  scheduleSave();
}

function undoDelete() {
  const restored = undoStack.pop();
  if (!restored) return;
  const idx = Math.min(restored.__originalIndex ?? tasks.length, tasks.length);
  delete restored.__originalIndex;
  tasks.splice(idx, 0, restored);
  selectedId = restored.id;
  render();
  scheduleSave();
}

function toggleComplete(id) {
  const task = tasks.find(t => t.id === id);
  if (!task) return;
  task.completed = !task.completed;
  render();
  scheduleSave();
}

function updateTitle(id, title) {
  const task = tasks.find(t => t.id === id);
  if (!task) return;
  task.title = title;
  scheduleSave();
}

function updateDescription(id, description) {
  const task = tasks.find(t => t.id === id);
  if (!task) return;
  task.description = description;
  scheduleSave();
}

function moveTask(fromId, toId, position) {
  const fromIdx = tasks.findIndex(t => t.id === fromId);
  if (fromIdx < 0 || fromId === toId) return;
  const [moved] = tasks.splice(fromIdx, 1);
  let toIdx = tasks.findIndex(t => t.id === toId);
  if (toIdx < 0) {
    tasks.push(moved);
  } else {
    if (position === 'after') toIdx += 1;
    tasks.splice(toIdx, 0, moved);
  }
  render();
  scheduleSave();
}

function deleteAll() {
  tasks = [];
  undoStack = [];
  hoveredId = null;
  selectedId = null;
  expandedId = null;
  editingTitleId = null;
  render();
  scheduleSave();
}

// --- rendering ------------------------------------------------------------

function render() {
  if (editingTitleId) return;

  listEl.innerHTML = '';
  for (const task of tasks) {
    listEl.appendChild(renderItem(task));
  }
  updateDeleteAllButton();

  if (expandedId) {
    const ta = listEl.querySelector('.description-edit');
    if (ta) {
      ta.focus();
      ta.setSelectionRange(0, 0);
    }
  }
}

function renderItem(task) {
  const li = document.createElement('li');
  li.className = 'task';
  li.dataset.id = task.id;
  li.draggable = true;
  if (task.completed) li.classList.add('completed');
  if (hoveredId === task.id) li.classList.add('hovered');
  if (selectedId === task.id) li.classList.add('selected');

  const checkbox = document.createElement('button');
  checkbox.type = 'button';
  checkbox.className = 'checkbox';
  checkbox.setAttribute('aria-label', task.completed ? 'Mark incomplete' : 'Mark complete');
  checkbox.addEventListener('click', (e) => {
    e.stopPropagation();
    toggleComplete(task.id);
  });
  li.appendChild(checkbox);

  const content = document.createElement('div');
  content.className = 'content';

  const titleEl = document.createElement('span');
  titleEl.className = 'title';
  titleEl.draggable = false;
  titleEl.textContent = task.title;
  titleEl.addEventListener('click', (e) => {
    e.stopPropagation();
    startEditTitle(task.id, titleEl);
  });
  content.appendChild(titleEl);

  if (task.description) {
    if (expandedId === task.id) {
      const ta = document.createElement('textarea');
      ta.className = 'description-edit';
      ta.draggable = false;
      ta.value = task.description;
      ta.rows = clampRows(task.description);
      ta.addEventListener('blur', () => commitDescriptionEdit(task.id, ta));
      ta.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
          e.preventDefault();
          ta.value = task.description;
          ta.blur();
        }
      });
      content.appendChild(ta);
    } else {
      const preview = document.createElement('span');
      preview.className = 'description-preview';
      preview.draggable = false;
      preview.title = task.description;
      preview.textContent = task.description;
      preview.addEventListener('click', (e) => {
        e.stopPropagation();
        expandedId = task.id;
        render();
      });
      content.appendChild(preview);
    }
  }

  li.appendChild(content);

  const del = document.createElement('button');
  del.type = 'button';
  del.className = 'delete';
  del.setAttribute('aria-label', 'Delete task');
  del.textContent = '\u00d7';
  del.addEventListener('click', (e) => {
    e.stopPropagation();
    deleteTask(task.id);
  });
  li.appendChild(del);

  li.addEventListener('mouseenter', () => {
    hoveredId = task.id;
    li.classList.add('hovered');
  });
  li.addEventListener('mouseleave', () => {
    if (hoveredId === task.id) {
      hoveredId = null;
      li.classList.remove('hovered');
    }
  });

  li.addEventListener('click', () => {
    if (selectedId !== task.id) {
      const prev = listEl.querySelector(`[data-id="${selectedId}"]`);
      if (prev) prev.classList.remove('selected');
      selectedId = task.id;
      li.classList.add('selected');
    }
  });

  li.addEventListener('dragstart', onDragStart);
  li.addEventListener('dragover', onLiDragOver);
  li.addEventListener('dragleave', onLiDragLeave);
  li.addEventListener('drop', onLiDrop);
  li.addEventListener('dragend', onDragEnd);

  return li;
}

function updateDeleteAllButton() {
  deleteAllBtn.disabled = tasks.length === 0;
}

// --- inline title edit ----------------------------------------------------

function startEditTitle(id, el) {
  const task = tasks.find(t => t.id === id);
  if (!task) return;
  editingTitleId = id;
  el.contentEditable = 'true';
  el.focus();
  selectAllText(el);

  const onInput = () => {
    const t = tasks.find(x => x.id === id);
    if (t) t.title = el.textContent;
  };
  const onBlur = () => {
    el.contentEditable = 'false';
    el.removeEventListener('input', onInput);
    el.removeEventListener('blur', onBlur);
    el.removeEventListener('keydown', onKey);
    editingTitleId = null;
    const t = tasks.find(x => x.id === id);
    if (!t) return;
    const newTitle = el.textContent.trim();
    if (!newTitle) {
      el.textContent = t.title;
      return;
    }
    t.title = newTitle;
    render();
    scheduleSave();
  };
  const onKey = (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      el.blur();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      const t = tasks.find(x => x.id === id);
      if (t) el.textContent = t.title;
      el.blur();
    }
  };

  el.addEventListener('input', onInput);
  el.addEventListener('blur', onBlur);
  el.addEventListener('keydown', onKey);
}

function selectAllText(el) {
  const range = document.createRange();
  range.selectNodeContents(el);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);
}

function commitDescriptionEdit(id, ta) {
  const task = tasks.find(t => t.id === id);
  if (!task) return;
  task.description = ta.value;
  expandedId = null;
  render();
  scheduleSave();
}

// --- new-task form --------------------------------------------------------

btnAddNote.addEventListener('click', () => {
  inputDesc.hidden = false;
  inputDesc.focus();
});

inputDesc.addEventListener('blur', () => {
  if (!inputDesc.value) inputDesc.hidden = true;
});

formEl.addEventListener('submit', (e) => {
  e.preventDefault();
  const title = inputTitle.value.trim();
  if (!title) {
    inputTitle.focus();
    return;
  }
  const description = inputDesc.value.trim();
  addTask(title, description);
  inputTitle.value = '';
  inputDesc.value = '';
  inputDesc.hidden = true;
  inputTitle.focus();
});

// --- delete-all with confirm ----------------------------------------------

deleteAllBtn.addEventListener('click', () => {
  if (tasks.length === 0) return;
  if (!deleteAllArmed) {
    deleteAllArmed = true;
    deleteAllBtn.textContent = 'Confirm delete';
    deleteAllBtn.classList.add('armed');
    deleteAllTimer = setTimeout(() => {
      deleteAllArmed = false;
      deleteAllBtn.textContent = 'Delete all';
      deleteAllBtn.classList.remove('armed');
      deleteAllTimer = null;
    }, DELETE_ALL_CONFIRM_MS);
  } else {
    if (deleteAllTimer) clearTimeout(deleteAllTimer);
    deleteAllTimer = null;
    deleteAllArmed = false;
    deleteAllBtn.textContent = 'Delete all';
    deleteAllBtn.classList.remove('armed');
    deleteAll();
  }
});

// --- hotkeys --------------------------------------------------------------

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    const ae = document.activeElement;
    if (ae && isTypingTarget(ae)) {
      e.preventDefault();
      ae.blur();
      return;
    }
  }

  if (isTypingTarget(e.target)) return;

  const cmd = e.metaKey || e.ctrlKey;

  if (cmd && (e.key === 'z' || e.key === 'Z')) {
    e.preventDefault();
    undoDelete();
    return;
  }

  if (cmd || e.altKey) return;

  switch (e.key) {
    case 'n':
    case 'N':
      e.preventDefault();
      inputTitle.focus();
      inputTitle.select();
      break;
    case 'c':
    case 'C': {
      const id = getActiveId();
      if (id) toggleComplete(id);
      break;
    }
    case 'd':
    case 'D': {
      const id = getActiveId();
      if (id) deleteTask(id);
      break;
    }
  }
});

// --- drag and drop --------------------------------------------------------

function onDragStart(e) {
  const id = e.currentTarget.dataset.id;
  draggingId = id;
  e.currentTarget.classList.add('dragging');
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', id);
  }
}

function onLiDragOver(e) {
  if (!draggingId) return;
  if (draggingId === e.currentTarget.dataset.id) return;
  e.preventDefault();
  e.stopPropagation();
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';

  clearDropIndicators();
  const rect = e.currentTarget.getBoundingClientRect();
  const before = e.clientY < rect.top + rect.height / 2;
  e.currentTarget.classList.add(before ? 'drop-before' : 'drop-after');
}

function onLiDragLeave(e) {
  e.currentTarget.classList.remove('drop-before', 'drop-after');
}

function onLiDrop(e) {
  if (!draggingId) return;
  e.preventDefault();
  e.stopPropagation();
  const targetId = e.currentTarget.dataset.id;
  const rect = e.currentTarget.getBoundingClientRect();
  const before = e.clientY < rect.top + rect.height / 2;
  clearDropIndicators();
  moveTask(draggingId, targetId, before ? 'before' : 'after');
  draggingId = null;
}

listEl.addEventListener('dragover', (e) => {
  if (!draggingId) return;
  if (e.target !== listEl) return;
  e.preventDefault();
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
  clearDropIndicators();
  const last = listEl.lastElementChild;
  if (last && last.dataset.id !== draggingId) {
    last.classList.add('drop-after');
  }
});

listEl.addEventListener('drop', (e) => {
  if (!draggingId) return;
  if (e.target !== listEl) return;
  e.preventDefault();
  const last = listEl.lastElementChild;
  if (last && last.dataset.id !== draggingId) {
    moveTask(draggingId, last.dataset.id, 'after');
  }
  clearDropIndicators();
  draggingId = null;
});

function onDragEnd(e) {
  e.currentTarget.classList.remove('dragging');
  clearDropIndicators();
  draggingId = null;
}

function clearDropIndicators() {
  for (const li of listEl.children) {
    li.classList.remove('drop-before', 'drop-after');
  }
}

// --- bootstrap ------------------------------------------------------------

load();
render();
inputTitle.focus();

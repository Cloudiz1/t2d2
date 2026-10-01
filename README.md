# Tasks

A single-page todo list. Plain HTML/CSS/JS, no build step, no backend.

Tasks persist in `localStorage`. They stay on whichever browser profile you used.

## Hotkeys

Hotkeys are ignored while typing in an input or textarea.

| Key | Action |
| --- | --- |
| `n` | Focus the new-task input |
| `c` | Toggle complete on the hovered (or selected) task |
| `d` | Delete the hovered (or selected) task |
| `Ctrl/Cmd+Z` | Undo the last delete (in-memory; cleared when you close the tab) |

## Deploy to Netlify (free, static tier)

Easiest path: open https://app.netlify.com/drop and drop this folder onto the page. Netlify gives you a URL immediately. No account required for a first deploy; if you skip account creation, the URL is anonymous and ephemeral — create a free account to keep it.

`netlify.toml` declares `publish = "."` so you can also connect a GitHub repo at https://app.netlify.com and get auto-deploys on push.

## Local development

Open `index.html` in a browser. Nothing to install.

## Browser caveat

Tasks live in `localStorage`. They are scoped to one browser profile on one device. Clearing site data, switching browsers, or using a private window will show an empty list. The undo stack lives only in memory and is lost when the tab closes — it is only meant to recover from an accidental `d` press within the same session.

# AGENTS.md — src/

**Role:** Vue 3 frontend — app entry, 4-view UI, Pinia state, Tauri bridge

## STRUCTURE

```
src/
├── main.ts              # Vue app entry: createApp + Pinia + Antd
├── App.vue              # Root: 4-view tabbed router
├── stores/              # Pinia stores (cross-view state)
│   ├── camera.ts        #   Device connection state
│   ├── capture.ts       #   Photo capture state machine
│   ├── template.ts      #   Template config + composite status
│   └── update.ts        #   OTA update state
├── views/               # Route-level views (one per screen)
│   ├── CaptureView.vue  #   Live MJPEG preview + capture trigger
│   ├── SelectionView.vue#   Photo grid + selection
│   └── PreviewView.vue  #   Template composite preview + print
├── components/          # Reusable components
│   ├── IdleView.vue     #   Standby/screensaver screen
│   ├── SettingsPanel.vue#   App settings modal
│   └── UpdateBanner.vue #   Update notification banner
├── styles/
│   └── tailwind.css     # Tailwind CSS 4 entry + global styles
└── env.d.ts             # Vite env type declarations
```

## WHERE TO LOOK

| Task | File |
|------|------|
| View routing logic | `App.vue` — tab-based `v-if` switching |
| Camera connection | `stores/camera.ts` — `connected`, `deviceId`, `running` |
| Capture flow | `stores/capture.ts` — mode, photos[], device state |
| Template config | `stores/template.ts` — current template selection |
| Update status | `stores/update.ts` — downloading/ready/applying |
| Live camera preview | `views/CaptureView.vue` — MJPEG `<img>` |
| Photo selection | `views/SelectionView.vue` — grid with selection |
| Composite + print | `views/PreviewView.vue` — preview + print button |
| Settings | `components/SettingsPanel.vue` — Ant Design modal |

## DATA FLOW

```
App.vue (tab controller)
  │
  ├─ idle ──── IdleView.vue
  │              │
  ├─ capture ── CaptureView.vue ─── captures → stores/capture.photos[]
  │                                      │
  ├─ selection ─ SelectionView.vue ──── reads photos[], selects → selected[]
  │                                      │
  └─ preview ── PreviewView.vue ──── sends selected[] to composite command
                                         │
                                         └─ renders result → print
```

All cross-view state is in Pinia stores. No prop drilling between views.

## CONVENTIONS

- **Composition API** everywhere (`<script setup lang="ts">`)
- **Pinia stores** for all shared state — no `provide/inject` or event bus
- **Tailwind CSS 4** for layout and spacing; Ant Design for form controls
- **Tauri invoke** for all backend commands (`@tauri-apps/api/core`)
- MJPEG preview: `<img>` with `http://127.0.0.1:27183` (multipart/x-mixed-replace)

## ANTI-PATTERNS

- No `as any` or `@ts-ignore` — use proper types
- No local state for cross-view data — belongs in Pinia
- No direct filesystem access — use Tauri commands

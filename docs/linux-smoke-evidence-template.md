# Linux Native Smoke Evidence Template

Copy this template and fill it out when submitting a Linux smoke report.

---

## Tester

- **Name or handle:**
- **Date:**
- **Artifact source:** (GitHub release / CI artifact / other)

## Environment

- **Distribution:**
- **Distribution version:**
- **Kernel:**
- **Architecture:**
- **Desktop environment:**
- **Display server:** (X11 / Wayland)
- **Session type:** (user session / remote / other)
- **WebKitGTK version:**
- **GTK version:**
- **FUSE/AppImage support:**
- **Machine type:** (physical / VM / cloud desktop)
- **GPU/graphics mode:** (hardware accelerated / software rendering)

## Artifact

- **Filename:**
- **SHA256 from checksums.txt:**
- **SHA256 locally computed:**
- **SHA256 match:** yes / no

## Smoke Steps

| Step | Result | Notes |
|------|--------|-------|
| chmod executable | pass / fail | |
| Launch from terminal | pass / fail | |
| App window opens | pass / fail | |
| Repository opens | pass / fail | |
| Roadmap/task UI loads | pass / fail | |
| Dashboard link opens | pass / fail | |
| Dashboard shows correct version | pass / fail | |
| No-key beta mode works | pass / fail | |
| Close and relaunch | pass / fail | |

## Logs

```text
Paste terminal output here after removing secrets.
```

## Screenshots

- **Main window:** (attach image)
- **Dashboard:** (attach image or describe)
- **Error dialog, if any:** (attach image)

## Result

**pass / fail / blocked**

## Notes

(Any additional observations, warnings, or suggestions)

---

**Reminder:** Do NOT share API keys, tokens, `.env` files, private repository contents, or screenshots containing secrets.

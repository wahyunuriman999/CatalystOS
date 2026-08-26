# CATALYST OS DESKTOP SHELL SPECIFICATION
**Version:** 1.0 (Real System Directive)

*This document contains the mandatory directives for implementing the Catalyst OS Desktop Shell. It was established by the Principal Architect to forbid static mockups and enforce real system engineering.*

## 1. VISUAL DIRECTION
- Futuristic, premium, minimal, immersive, dark glassmorphism.
- Translucent panels, subtle blur, depth, soft neon illumination (blue/purple/cyan accent).
- Smooth animations, spatial layout, high information density.
- **Goal:** macOS-level polish + Windows-level productivity + Linux-level control.

## 2. CRITICAL REQUIREMENT — REAL SYSTEM, NOT MOCKUP
- Every visible desktop element MUST correspond to a real system capability.
- No fake hardcoded values. If an API is not implemented, leave it empty or build the abstraction.
- The desktop must react to real system state.

## 3. NO AI ASSISTANT PANEL
- Remove the AI Assistant / CORA panel.
- Focus strictly on applications, files, windows, system state, productivity, navigation.

## 4. DESKTOP STRUCTURE
- **Top System Bar**: Logo, active app menus, real clock (RTC), real system status.
- **Left Navigation Rail**: Icons for Home, Apps, Files, Terminal, Settings (MUST map to real apps).
- **Main Desktop**: Real dynamic clock, Unified System Search.
- **Favorite Apps**: Discovered from actual application registry.
- **Workspaces**: Real context switching, shortcuts (Super+1, etc).
- **Right Information Panel**: Real calendar, real notifications (from system/apps).
- **Bottom Dock**: Real running apps, pinned apps, active window indicators.
- **System Monitor / Storage**: Real hardware metrics API.
- **Window Management**: P0 feature. Real drag, drop, minimize, maximize.

## 5. REAL DATA CONTRACT
All widgets must follow a strict data contract:
`Widget -> System API -> Kernel/Hardware`
Never hardcode fake data.

## 6. IMPLEMENTATION PRIORITY
- **P0 — Desktop Foundation**: Desktop Shell, Window Manager, Top Bar, Dock, App Launcher, Workspace system, Input handling (Mouse/KB).
- **P1 — Core System Integration**: RTC, network, battery, CPU/RAM, storage, notifications.
- **P2 — Core Applications**: File Manager, Terminal, Settings, SysMon.
- **P3 — Advanced UX**: Search, workspace transitions, window snapping, dynamic panels.
- **P4 — Polish**: GPU rendering, animations, optimizations.

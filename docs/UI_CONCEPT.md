<!-- CATALYST OS — PROPRIETARY AND CONFIDENTIAL -->
<!-- Copyright (c) 2024-2026 Wahyu Nur Iman. All rights reserved. -->

# Catalyst OS UI/UX Concept (Mandate)

## Core Design Philosophy
The Desktop Environment for Catalyst OS must strictly adhere to the futuristic, modern, "glassmorphic" concept provided by the user on 26 August 2026. This is the absolute visual target for the GUI.

## Visual Elements (From Mockup)
1. **Theme:** Dark mode, sci-fi aesthetic, deep blues and purples. Space/planet landscape wallpaper.
2. **Material:** Heavy use of Glassmorphism. Panels and widgets must have semi-transparent backgrounds (e.g., `rgba(20, 20, 40, 0.6)`) with background blur (acrylic effect) and subtle, bright borders (1px solid `rgba(255, 255, 255, 0.1)`).
3. **Typography:** Clean, modern sans-serif (e.g., Inter, Roboto, or SF Pro). 

## Desktop Layout
- **Top Bar (Status Bar):**
  - Left: OS Logo (C), File, Edit, View, Go, Tools, Help.
  - Center: Date and Time ("Tue, 25 Aug 2026 10:42 AM").
  - Right: System tray icons (WiFi, Network, Volume, Battery 100%, User Avatar).
- **Left Sidebar (Dashboard):**
  - Greeting: "Selamat pagi, Wahyu 👋".
  - System Overview: Circular CPU/RAM/GPU/NPU usage charts.
  - Storage & Network stats.
  - Focus Mode toggle.
  - AI Assistant (CORA) widget with voice wave animation.
  - Media Player widget (Now Playing).
  - Far-left icon rail (Home, Dashboard, Code, Files, Mail, Calendar, Settings).
- **Center Area (Workspace):**
  - Large digital clock widget (Time, Date, Weather).
  - Floating Search Bar ("Cari aplikasi, file, atau perintah...").
  - App Launcher ("Aplikasi Favorit"): Catalyst Terminal, Code Studio, File Manager, Relay Chat, Design Hub, Media Studio, System Monitor, AI Playground.
  - Active Workspaces ("Ruang Kerja Aktif"): Cards for active projects.
- **Right Sidebar (Info Center):**
  - Interactive Calendar widget.
  - Agenda widget (Daily Standup, Review, Focus Time).
  - Notifications feed.
  - Data Vault widget (Encrypted storage status).
- **Bottom Dock:**
  - Floating macOS-style dock with glowing active indicators.
  - Contains icons for core apps and Trash.

## Engineering Requirements for Implementation
To achieve this in a bare-metal kernel, we must build:
1. A **Double-Buffered 2D Compositor** running in the kernel (or a dedicated Ring 3 server).
2. An **Alpha-Blending Engine** to support transparency and gradients.
3. A **TrueType Font (TTF) Rasterizer** (or use high-res bitmap fonts initially).
4. **CatFS** to load the wallpaper image (PNG/JPG decoder) and icons from disk.
5. **Mouse & Keyboard Drivers** (PS/2 & USB HID) for interacting with the dock and widgets.

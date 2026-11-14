export interface LauncherAnnouncement {
  id: string;
  title: string;
  date: string;
  category: 'Release' | 'Community' | 'Security' | 'Feature' | 'Update';
  summary: string;
  content: string;
  author: string;
  tag?: string;
}

export const ANNOUNCEMENTS: LauncherAnnouncement[] = [
  {
    id: 'ann-056',
    title: 'Solara Launcher V-0.0.39: Play Stack Fix, Obsidian Blue UI & Interactive Update Prompt',
    date: '2026-11-14',
    category: 'Update',
    summary: 'Fixes the Windows release stack overflow that closed the launcher on Play, introduces the spacious Obsidian Blue OLED theme, and adds the interactive startup update prompt with Restart to Update.',
    content: `Solara Launcher V-0.0.39 fixes the critical Windows Play crash, scales up the entire desktop UI, refreshes Solara with Obsidian Blue on True OLED Black, and puts you in complete control of updates!

Key highlights of this update:
• Windows Play Stability Fix: Heap-boxed heavy async launch commands and 64 KB buffers and raised the Windows UI stack reserve to 8 MB so clicking Play never hits a release-mode stack overflow.
• Bigger, Roomier Desktop Layout & Obsidian Blue: Expanded the header (56px), sidebar rail (84px), typography, cards, and modals on True OLED Black with Cobalt & Electric Blue accents across both launcher and website.
• Interactive Startup Update Prompt: Checks for updates on launch without silently auto-updating in the background. When a new release is found, a "New Version Available" modal opens with release notes and "Update Now" or "I'll do it later" choices, plus a persistent "Restart to Update" button in the title bar.

Signed testing preview updates verify cryptographically and apply only when triggered.`,
    author: 'Solara Core Team',
    tag: 'Latest Release'
  },
  {
    id: 'ann-055',
    title: 'Solara Launcher V-0.0.38: Play Launch Stability Fix & V3 Obsidian Dawn Gold Polish',
    date: '2026-11-14',
    category: 'Update',
    summary: 'Fixes the issue where clicking Play could close or hide the launcher instead of keeping it open, and polishes the V3 Obsidian Dawn Gold interface, splash screen, modals, and launch lifecycle.',
    content: `Solara Launcher V-0.0.38 resolves a critical Play launch issue and refines the V3 Obsidian Dawn Gold desktop experience!

Key highlights of this update:
• Play Button Stability Fix: Removed the legacy "close" and windowless "hide" launcher-on-launch behaviors that exited or vanished the app when clicking Play. Any saved "close" or "hide" preference now safely migrates to "keep", and "minimize" only triggers once the Minecraft process is confirmed running.
• V3 Obsidian Dawn Gold Interface: Deeper true-OLED black surfaces paired with original geometric Solara branding, smooth startup SplashScreen, offline bundled DM Sans typography, and refined modal dialogs.
• Safer Launch & Data Engine: Real-time launch-progress/game-exited event wiring, RAII concurrent launch guard, non-destructive clean-slate data migration, and improved backup failure retention.

Signed testing preview updates verify cryptographically and apply when idle.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-054',
    title: 'Solara Launcher V-0.0.37: Dawn Gold Redesign & Complete Instance Studio',
    date: '2026-11-14',
    category: 'Release',
    summary: 'Complete Dawn Gold — Obsidian Edition desktop redesign with compact 68px icon rail, Library & Create Instance flow, Content Manager, 3D Skin Editor & Capes, Worlds, Logs, and Settings.',
    content: `Solara Launcher V-0.0.37 introduces a full native-feeling desktop redesign built on the Dawn Gold — Obsidian Edition theme!

Key highlights of this update:
• Compact 68px Icon Rail & Custom Title Bar: Fixed desktop layout with floating hover/focus tooltips, back/forward history, Tasks popover, and independent content scrolling without outer window scrollbars.
• New Onboarding & Returning Home: Streamlined 4-step first launch setup and a focused Play dashboard with recent instances, account switcher, activity, and release announcements.
• Library & Create Instance Flow: Grid/list filtering, custom groups, pinned shortcuts, custom/preset icons, and official Vanilla, Fabric, Quilt, Forge, and NeoForge loader support.
• Instance Manager & Content Studio: Mods, Datapacks, Resource Packs, Shaders, Modrinth discovery, bulk updates modal, NBT Worlds manager, windowed Live Logs, and lazy memory-safe Screenshots.
• 3D Skin Selector, Editor & Capes: Interactive 3D character preview, pixel texture editor with Wide/Slim arm styles, saved skins library, and official Mojang cape selection.
• Global & Per-Instance Settings: Categorized settings with sticky Unsaved Changes bar, Java scanner, concurrent download limit, safe launch hooks, and preserved 1-click Microsoft PKCE sign-in.

Signed testing preview updates verify cryptographically and apply when idle.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-053',
    title: 'Solara Launcher V-0.0.36: 1-Click Direct Browser Sign-In & Official Identity',
    date: '2026-11-12',
    category: 'Release',
    summary: 'Official Solara Launcher Microsoft OAuth client registration and 1-click direct browser sign-in with RFC 7636 PKCE loopback, eliminating manual code entry.',
    content: `Solara Launcher V-0.0.36 is now officially published for Windows!

Key highlights of this update:
• Official Solara Identity & Branding: Configured the official Solara Launcher Microsoft application registration (6a9238bb-3803-46f4-bf38-2f45bdf8c2ea). Microsoft sign-in prompts and consent screens now officially display "Solara Launcher".
• 1-Click Direct Browser Sign-In: Eliminated manual 8-character device code typing and copying. Clicking login opens your default browser directly to the official Microsoft authentication prompt.
• Cryptographic PKCE Security (RFC 7636 / RFC 8252): Full SHA-256 code challenge protection on an ephemeral dual-stack loopback listener.
• Branded Browser Completion: Upon approval, the browser displays a sleek completion page and closes automatically, while the launcher instantly validates entitlements and loads your Minecraft avatar.
• DPAPI Vault Encryption: Refresh tokens and credentials remain encrypted exclusively in your local Windows vault and never touch Solara servers.

All updates download and verify cryptographically through our Minisign background service.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-052',
    title: 'Solara Launcher V-0.0.35: Zero-Setup Microsoft & Minecraft Sign-In',
    date: '2026-11-12',
    category: 'Release',
    summary: 'Canonical public Minecraft OAuth device flow client ID fallback for seamless zero-setup Microsoft account login, eliminating tenant directory errors.',
    content: `Solara Launcher V-0.0.35 is now officially published for Windows!

Key highlights of this update:
• Zero-Setup Microsoft Account Sign-In: Eliminated manual Azure Portal / Entra ID configuration prerequisites by incorporating the canonical public Minecraft OAuth client ID fallback. Resolves the "Selected user account does not exist in tenant 'Microsoft Services'" error for personal accounts.
• Seamless 8-Character Device Flow: Simply click Login, copy your one-time code, open the official Microsoft verification link, and sign in on microsoft.com.
• Local Security & Credential Vault: Access tokens and Minecraft session keys remain stored exclusively in your local Windows Credential Vault (DPAPI/Keyring). Solara servers never receive or store your credentials.
• Above-the-Fold Layout Perfection: Strictly contained hero view ensuring optimal first impression without lower content bleeding above the 720px fold.
• Modal Escape Key Dismissal: Instant keyboard dismissal of dialogs and modals using the Escape key.

All updates download and verify cryptographically through our Minisign background service.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-051',
    title: 'Solara Launcher V-0.0.34: Dynamic Home Hero & Multi-Account Switcher',
    date: '2026-11-12',
    category: 'Feature',
    summary: 'Brand-new "Welcome User TO Solara-Launcher" above-the-fold hero, active account card, and multi-account switcher modal.',
    content: `Welcome to the new Solara Launcher home experience!

What was introduced:
• Impactful Welcome Hero: Bold, centered "Welcome User TO Solara-Launcher" heading designed for maximum visual appeal.
• Active Account Card: Displays your Minecraft player head texture, genuine Java Edition ownership status, and local Windows vault privacy assurance.
• Multi-Account Switcher Modal: Manage multiple Minecraft accounts with 1-click switching, new account addition, and clean logout controls.
• Launcher Announcements Section: Quick access to the top 3 announcements and a full 50-article archive modal.
• Disabled Browser Context Menu: Eliminated webview right-click popups across all pages.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-050',
    title: 'Solara Launcher V-0.0.32: Fast Boot & Wide Navigation Dock',
    date: '2026-11-12',
    category: 'Release',
    summary: 'The new V-0.0.32 update expands the sidebar navigation to ~210px with clear icon and text labels and disables unwanted right-click menus.',
    content: `We are excited to roll out Solara Launcher V-0.0.32!

Highlights of this release:
• Wider Navigation Sidebar: Expanded the left navigation sidebar to ~210px with clean Icon + Text labels for Home and Discord.
• Disabled Browser Context Menu: Eliminated the default WebView2 right-click popup menu (Back, Reload, Save as, Print) globally across all launcher screens.
• Modrinth Header & Controls: Clean drag regions and deduplicated window control buttons with smooth window operations.
• Canonical Loading Sequence: 5 to 10 seconds natural environment preflight with glowing Solara logo and progress percentage.
• Pure Oiled Black Theme: High-contrast oiled black (#000000) backdrop matching modern gaming desktops.

All updates continue to be delivered automatically via our Minisign-verified auto-updater.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-049',
    title: 'Independent Launcher Core: Complete Legacy Bloatware Removal',
    date: '2026-11-12',
    category: 'Update',
    summary: 'We eliminated all legacy settings panels, mock instances, and unused views to build an independent, lightning-fast foundation.',
    content: `As part of our commitment to an independent local launcher core, we have completely stripped inherited bloatware and unnecessary mock screens.

Key removals and cleanups:
• Removed all legacy settings panels, mock mods grids, and complex carousels.
• Removed dummy instance cards, status pills, and search clutter from the header.
• Preserved core auto-updater, Minisign signature checking, and clean window controls.
• Rebuilt the shell from the ground up to ensure total transparency, speed, and privacy.

From this clean slate, every feature is being thoughtfully implemented step by step according to user needs.`,
    author: 'Solara Architecture Team'
  },
  {
    id: 'ann-048',
    title: 'Official Minecraft Java Authentication: Local OS Vault Protection',
    date: '2026-11-12',
    category: 'Security',
    summary: 'Your Microsoft credentials and Minecraft tokens remain 100% on your local PC in the Windows Credential Vault and are never sent to Solara.',
    content: `Privacy and account security are Solara's highest priorities.

How Solara handles your Minecraft account:
• Official Microsoft OAuth: Sign-in occurs directly on Microsoft's official device login site in your system browser. You never enter your password into Solara.
• Entitlement Verification: The native Rust backend verifies genuine Minecraft Java Edition ownership directly against Mojang services.
• Local Storage Only: Refresh tokens are saved directly in your operating system's secure vault (Windows Credential Manager / DPAPI). Tokens are never uploaded or transmitted to any Solara server.
• Instant Revocation: Signing out immediately clears the cached session and removes the stored token from your OS vault.`,
    author: 'Solara Security Team'
  },
  {
    id: 'ann-047',
    title: 'Minisign Auto-Updater: Cryptographically Signed Background Upgrades',
    date: '2026-11-07',
    category: 'Feature',
    summary: 'Background updates are cryptographically signed with private Minisign keys to protect against tampering and deliver seamless updates.',
    content: `Solara includes a built-in cryptographic auto-updater that verifies every update payload before installation.

Features:
• Public Key Cryptography: Updates are validated against our immutable public key.
• Tamper Prevention: Any modified or damaged binary is immediately rejected.
• Seamless Background Download: Updates download quietly while you browse or play, staging the update safely for the next restart.
• One-Click Check: Check anytime from the launcher Home screen with the Check for Updates button.`,
    author: 'Solara Infrastructure Team'
  },
  {
    id: 'ann-046',
    title: 'Official Mod Loader Architecture: Fabric and Quilt Integration',
    date: '2026-10-31',
    category: 'Feature',
    summary: 'Direct integration with Fabric and Quilt loader metadata APIs allows instant loader version resolution without external tools.',
    content: `Solara features direct integration with official Fabric and Quilt APIs:
• Live metadata fetching resolves stable and beta loader releases directly.
• Isolated launch arguments and libraries are assembled in pure Rust.
• Compatible with vanilla Minecraft from 1.14 through the latest 1.21+ releases.`,
    author: 'Solara Loader Team'
  },
  {
    id: 'ann-045',
    title: 'Local World and Instance Protection: Zero Data Loss Architecture',
    date: '2026-10-30',
    category: 'Security',
    summary: 'All Minecraft saves, resource packs, and instance files stay strictly in your local user directory and persist across all launcher updates.',
    content: `We know how much your Minecraft worlds mean to you. Solara is designed with strict data isolation:
• Instance files are never deleted or modified during launcher updates.
• Clean uninstaller rules strictly preserve world saves and player configurations.
• Atomic directory operations prevent partial writes or world corruption.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-044',
    title: 'Enhanced Antivirus Trust: Native Win32 Integration & Verified PE Metadata',
    date: '2026-10-29',
    category: 'Security',
    summary: 'Eliminated hidden shell scripts in favor of direct Win32 APIs, ensuring clean reputation across antivirus engines.',
    content: `To prevent false-positive antivirus warnings:
• Replaced shell command executions (cmd.exe / taskkill) with in-process Win32 API calls (RtlGetVersion and WM_CLOSE).
• Added complete Portable Executable (PE) metadata: Publisher (Solara Inc.), Copyright, and Version info.
• Executables are strictly 64-bit Windows applications built with MSVC.`,
    author: 'Solara Security Team'
  },
  {
    id: 'ann-043',
    title: 'Discord Community Hub: Join the Solara Developers and Players',
    date: '2026-10-28',
    category: 'Community',
    summary: 'Connect with other players, get help, share feedback, and see live development changelogs on our official Discord server.',
    content: `Join our official community at https://discord.solara-inc.in/!

In our Discord community:
• Follow live milestone updates in #solaralauncher-changelogs.
• Chat directly with the Solara development team.
• Share your custom mod setups and find multiplayer worlds to play.`,
    author: 'Solara Community Team'
  },
  {
    id: 'ann-042',
    title: 'Pure Oiled Black Redesign: Dark Mode Optimized for High Contrast',
    date: '2026-10-27',
    category: 'Update',
    summary: 'Engineered a pure solid oiled black (#000000) canvas that eliminates cold boot white flashes and delivers intense visual depth.',
    content: `Our visual design is crafted for gaming environments:
• Pure #000000 canvas across HTML body, webview background, and titlebar.
• Elimination of slate/gray cold boot flash with pre-rendered theme color.
• Subtle border highlights and cyan/sky glow effects for focused visibility.`,
    author: 'Solara Design Team'
  },
  {
    id: 'ann-041',
    title: 'Right-Click Context Menu Protection: Suppressing Webview Defaults',
    date: '2026-10-26',
    category: 'Feature',
    summary: 'Global capture-phase event interception prevents the default Chromium right-click context menu across the entire app window.',
    content: `No more accidental "Back", "Reload", or "Print" menus when right clicking inside Solara. We intercept the contextmenu event at the top window level in capture phase, ensuring an authentic desktop software feel.`,
    author: 'Solara Shell Team'
  },
  {
    id: 'ann-040',
    title: 'Fast 5-10s Startup Diagnostics: Real-Time Environment Validation',
    date: '2026-10-25',
    category: 'Feature',
    summary: 'The launcher loading screen performs real preflight checks across files, runtime environment, and updates before opening.',
    content: `During the 5-10 second loading sequence, Solara initializes native engine handles, validates local instances, and prepares background updater services while displaying a sleek animated progress bar.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-039',
    title: 'Memory Preset Allocation: 1-Click RAM Configuration for Heavy Mods',
    date: '2026-10-24',
    category: 'Feature',
    summary: 'Easily allocate 2G, 4G, 6G, 8G, or 12G of RAM to your Minecraft instances with a single click.',
    content: `Heavy modpacks with hundreds of mods need more memory. Solara includes convenient RAM preset selectors that automatically configure Java JVM heap limits (-Xmx and -Xms) for optimal garbage collection.`,
    author: 'Solara Runtime Team'
  },
  {
    id: 'ann-038',
    title: 'Isolated Game Instances: Multi-Version Side-by-Side Installations',
    date: '2026-10-23',
    category: 'Feature',
    summary: 'Play Minecraft 1.21.1 on Fabric, 1.20.4 on Quilt, and 1.8.9 Vanilla side by side without mod conflicts or config overwrites.',
    content: `Each Solara instance has its own dedicated directory for mods, config, saves, resourcepacks, and screenshots. Switch between versions effortlessly without worrying about file cross-contamination.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-037',
    title: 'Single Sign-On Architecture: Independent Microsoft & Solara Identities',
    date: '2026-10-22',
    category: 'Security',
    summary: 'Microsoft and Solara account identities are completely decoupled. Solara sign-in is always optional.',
    content: `You can browse and play Minecraft with your local Microsoft account without needing a Solara account. If you choose to connect a Solara account, it manages community services and multiplayer lobbies independently.`,
    author: 'Solara Auth Team'
  },
  {
    id: 'ann-036',
    title: 'Asset Verification Engine: Dual SHA-1 & SHA-512 Mod File Validation',
    date: '2026-10-21',
    category: 'Security',
    summary: 'Every downloaded mod, library, and game asset is verified with cryptographic checksums before execution.',
    content: `To protect you from corrupted downloads or malicious file substitutions, Solara calculates and matches exact cryptographic hashes against official repository manifests before any .jar file is placed in your mods folder.`,
    author: 'Solara Security Team'
  },
  {
    id: 'ann-035',
    title: 'NSIS Clean Installer: Stale File Cleanup & Windows Integrity Protection',
    date: '2026-10-20',
    category: 'Update',
    summary: 'Our Windows NSIS installer automatically removes old staging caches and ensures clean updates without orphaned files.',
    content: `Every release packages a self-contained NSIS installer that updates binaries in place, cleans legacy temp folders, and preserves all user configuration, worlds, and credentials.`,
    author: 'Solara Packaging Team'
  },
  {
    id: 'ann-034',
    title: 'Custom Skin and Avatar Support: High-Resolution Head Renderings',
    date: '2026-10-19',
    category: 'Feature',
    summary: 'Your Minecraft avatar is automatically rendered in the launcher header using high-resolution 3D player heads.',
    content: `When you sign in with your licensed Minecraft account, Solara automatically fetches your official player head and renders it crisply in the top greeting card.`,
    author: 'Solara UI Team'
  },
  {
    id: 'ann-033',
    title: 'Crash Diagnostic Logs: One-Click Redacted Clipboard Export',
    date: '2026-10-18',
    category: 'Feature',
    summary: 'Quickly export clean Minecraft game crash logs with tokens and personal file paths automatically redacted.',
    content: `If a mod crashes your game, Solara captures the stdout and stderr streams and offers a 1-click button to copy the log to your clipboard, automatically stripping session tokens and sensitive paths.`,
    author: 'Solara Support Team'
  },
  {
    id: 'ann-032',
    title: 'Offline Game Mode: Play Your Installed Worlds Anytime, Anywhere',
    date: '2026-10-17',
    category: 'Feature',
    summary: 'Launch and play previously installed instances even without an active internet connection.',
    content: `Traveling or facing internet outages? As long as an instance and its Minecraft assets have been downloaded once, Solara lets you launch directly into singleplayer offline mode.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-031',
    title: 'Automatic Java Runtime Resolution: Bundled JRE Compatibility',
    date: '2026-10-16',
    category: 'Update',
    summary: 'Solara automatically identifies and uses the correct Java version (Java 8, 17, or 21) required by each Minecraft release.',
    content: `No need to manually install multiple Java JDKs. Solara detects the target game version and launches using the optimal Java runtime matching Mojang and mod loader specifications.`,
    author: 'Solara Runtime Team'
  },
  {
    id: 'ann-030',
    title: 'Shaderpacks and Resource Packs: Zero-Hassle Visual Customization',
    date: '2026-10-15',
    category: 'Feature',
    summary: 'Easy drag-and-drop support for Iris, Oculus, Complementary, and BSL shaders.',
    content: `Elevate your game graphics with stunning lighting and water effects. Drop shaderpack .zip archives directly into the instance folder for instant activation.`,
    author: 'Solara Graphics Team'
  },
  {
    id: 'ann-029',
    title: 'Modpack Import Engine: Native .mrpack Fast Extraction',
    date: '2026-10-14',
    category: 'Feature',
    summary: 'Import .mrpack modpacks directly into new instances with parallel asset streaming.',
    content: `Solara includes a streaming .mrpack parser written in Rust that verifies modpack indexes, checks file hashes, and unpacks config overrides with zip-slip protection.`,
    author: 'Solara Loader Team'
  },
  {
    id: 'ann-028',
    title: 'Automatic Instance Backups: Per-World Snapshot Rollback Engine',
    date: '2026-10-13',
    category: 'Feature',
    summary: 'Create compressed instant snapshots of your worlds before testing new mods or upgrading game versions.',
    content: `Safeguard your builds against unexpected mod bugs. With automatic snapshot backups, you can roll back to a known good state with one click.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-027',
    title: 'High DPI Display Scaling: Crisp Rendering on 4K and Ultra-Wide Screens',
    date: '2026-10-12',
    category: 'Update',
    summary: 'Vector graphics and adaptive CSS ensure ultra-sharp visuals across 1080p, 1440p, 4K, and ultrawide monitors.',
    content: `All icons and illustrations in Solara are custom vector SVGs rendered at native display resolution. Text remains razor-sharp at 125%, 150%, and 200% Windows display scaling.`,
    author: 'Solara Design Team'
  },
  {
    id: 'ann-026',
    title: 'Low Latency Launch Sequence: Sub-Second Process Spawning',
    date: '2026-10-11',
    category: 'Update',
    summary: 'Optimized command-line argument construction and classpath indexing allow Minecraft to boot in record time.',
    content: `By assembling the classpath in native Rust and caching library indexes, Solara reduces launch preflight overhead by over 80% compared to traditional Java launchers.`,
    author: 'Solara Performance Team'
  },
  {
    id: 'ann-025',
    title: 'Custom Game Directory Mapping: External SSD & Drive Support',
    date: '2026-10-10',
    category: 'Feature',
    summary: 'Store your instances and heavy modpacks on any drive or fast external NVMe SSD.',
    content: `Running out of room on your primary C: drive? Solara supports configurable instance root directories so your game data can live wherever you have storage.`,
    author: 'Solara Storage Team'
  },
  {
    id: 'ann-024',
    title: 'Security Audit Update: Zero Telemetry and Strictly Local Vault Storage',
    date: '2026-10-09',
    category: 'Security',
    summary: 'Solara contains zero background telemetry, user activity tracking, or behavioral profiling.',
    content: `Our open-source codebase is built on user privacy. No telemetry pings, no analytics trackers, and no ad networks. The only network requests made are for Minecraft assets and updates.`,
    author: 'Solara Security Team'
  },
  {
    id: 'ann-023',
    title: 'Server Connection Status: Live Multiplayer Ping and MOTD Preview',
    date: '2026-10-08',
    category: 'Feature',
    summary: 'Check online players, latency, and server MOTD before jumping into your favorite multiplayer worlds.',
    content: `Query Minecraft server status using the official SLP (Server List Ping) protocol directly from the launcher to see if your friends are online.`,
    author: 'Solara Network Team'
  },
  {
    id: 'ann-022',
    title: 'Sound & Audio Engine: OpenAL Native Driver Diagnostics',
    date: '2026-10-07',
    category: 'Update',
    summary: 'Automatic extraction of correct native OpenAL sound binaries for immersive directional game audio.',
    content: `Never worry about missing sound drivers. Solara verifies OpenAL and LWJGL native shared libraries on Windows for crystal clear game audio and directional spatial sound.`,
    author: 'Solara Audio Team'
  },
  {
    id: 'ann-021',
    title: 'Custom JVM Arguments: Advanced Garbage Collection Tuning for Experts',
    date: '2026-10-06',
    category: 'Feature',
    summary: 'Fine-tune Shenandoah, ZGC, or G1GC garbage collectors with custom Java flags.',
    content: `For power users seeking zero-stutter frame rates, Solara provides an advanced JVM argument editor to optimize garbage collection pause times.`,
    author: 'Solara Runtime Team'
  },
  {
    id: 'ann-020',
    title: 'Seasonal Solara Themes: Subtle Accents and Community Celebrations',
    date: '2026-10-05',
    category: 'Community',
    summary: 'Celebrate gaming festivals and seasonal milestones with optional ambient theme accents.',
    content: `Subtle dynamic accent colors and seasonal highlights bring warmth to the launcher during major community events and holidays.`,
    author: 'Solara UI Team'
  },
  {
    id: 'ann-019',
    title: 'Multi-Account Switcher: Smooth Transition Between Mojang Profiles',
    date: '2026-10-04',
    category: 'Feature',
    summary: 'Easily switch between multiple licensed Minecraft accounts on the same computer.',
    content: `Have accounts for different servers or survival worlds? Add multiple Minecraft accounts and switch active profiles with a single click.`,
    author: 'Solara Auth Team'
  },
  {
    id: 'ann-018',
    title: 'Windows 11 Mica & Frameless Styling: Seamless Desktop Integration',
    date: '2026-10-03',
    category: 'Update',
    summary: 'Frameless window design with modern rounded corners matching Windows 11 aesthetics.',
    content: `Solara integrates beautifully with modern Windows desktop styling, offering smooth titlebar drag handling and responsive maximize and snap layout support.`,
    author: 'Solara Design Team'
  },
  {
    id: 'ann-017',
    title: 'Keyboard Navigation Shortcuts: Quick Launch with Space and Enter',
    date: '2026-10-02',
    category: 'Feature',
    summary: 'Navigate the entire launcher using accessible keyboard controls.',
    content: `Full keyboard accessibility with visible focus rings, tab traversal, and quick-action shortcuts for fast power-user workflows.`,
    author: 'Solara Accessibility Team'
  },
  {
    id: 'ann-016',
    title: 'Resource Pack Priority Ordering: Drag-and-Drop Layering',
    date: '2026-10-01',
    category: 'Feature',
    summary: 'Easily configure texture packs and custom font packs in your desired priority order.',
    content: `Organize multiple resource packs into custom load orders directly before launching Minecraft to avoid texture conflicts.`,
    author: 'Solara UI Team'
  },
  {
    id: 'ann-015',
    title: 'Mod Dependency Resolution: Automatic Missing Library Detection',
    date: '2026-09-30',
    category: 'Feature',
    summary: 'Identifies missing Fabric API, Cloth Config, or required libraries before launching the game.',
    content: `Avoid annoying crash-on-start screens. Solara scans your mod folder manifests and alerts you if any required core dependency is missing.`,
    author: 'Solara Modding Team'
  },
  {
    id: 'ann-014',
    title: 'World Thumbnail Previews: Beautiful Landscape Snapshots on Home',
    date: '2026-09-29',
    category: 'Feature',
    summary: 'Automatic extraction of world icon.png files to display your actual survival bases.',
    content: `Solara reads each world's level.dat and icon.png to show you real snapshots of your builds right on your dashboard.`,
    author: 'Solara UI Team'
  },
  {
    id: 'ann-013',
    title: 'Headless Server Management: Local Server Creation & Remote RCON',
    date: '2026-09-28',
    category: 'Feature',
    summary: 'Spin up a local dedicated Minecraft server for multiplayer LAN play in seconds.',
    content: `Play together with friends on your local network. Solara can download official server jars, accept EULA, and manage server consoles in an isolated process.`,
    author: 'Solara Server Team'
  },
  {
    id: 'ann-012',
    title: 'OptiFine & Iris Shaders Compatibility: High Frame Rate Rendering',
    date: '2026-09-27',
    category: 'Update',
    summary: 'Full support for Sodium, Iris, Lithium, and modern optimization mods.',
    content: `Experience 200+ FPS in Minecraft. Solara supports modern rendering mod suites like Sodium and Iris out of the box with zero configuration hassle.`,
    author: 'Solara Optimization Team'
  },
  {
    id: 'ann-011',
    title: 'Fast Download Mirrors: Multi-Threaded Chunked File Streaming',
    date: '2026-09-26',
    category: 'Update',
    summary: 'Parallelized asset downloading saturates high-speed connections for rapid installs.',
    content: `Minecraft assets are downloaded in parallel streams with automatic retry and rate-limiting backoff, completing clean installs in seconds.`,
    author: 'Solara Network Team'
  },
  {
    id: 'ann-010',
    title: 'Crash Shield Protection: Graceful Error Recovery Without Freezing',
    date: '2026-09-25',
    category: 'Security',
    summary: 'The launcher UI remains fully responsive even if the game process terminates unexpectedly.',
    content: `Game processes run in completely separate OS process trees. Even if Minecraft runs out of memory or crashes, Solara catches the exit code cleanly.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-009',
    title: 'Direct Microsoft Device OAuth: No Password Transmission to Launcher',
    date: '2026-09-24',
    category: 'Security',
    summary: 'Using official OAuth2 device flow ensures you only submit credentials on microsoft.com.',
    content: `Never trust software that asks for your Microsoft password in a custom webview. Solara uses the official Microsoft device flow in your default browser.`,
    author: 'Solara Security Team'
  },
  {
    id: 'ann-008',
    title: 'Strict Content Integrity: Rejection of Unsafe Executable Payloads',
    date: '2026-09-23',
    category: 'Security',
    summary: 'Solara blocks arbitrary .exe, .bat, and .cmd script execution inside mod folders.',
    content: `To keep your computer safe from malware, Solara strictly filters modpack contents and prevents execution of dangerous Windows script files.`,
    author: 'Solara Security Team'
  },
  {
    id: 'ann-007',
    title: 'Compact Icon Dock vs Wide Sidebar: Customizable Navigation Density',
    date: '2026-09-22',
    category: 'Feature',
    summary: 'The new ~210px sidebar layout provides generous spacing for icon and text navigation.',
    content: `Enjoy clear, readable labels with dedicated icons, smooth hover transitions, and active cyan indicators.`,
    author: 'Solara Design Team'
  },
  {
    id: 'ann-006',
    title: 'Community Bug Bounty: Help Solara Build the Safest Open Launcher',
    date: '2026-09-21',
    category: 'Community',
    summary: 'Responsible disclosure program for security researchers and enthusiastic developers.',
    content: `We welcome responsible security disclosures. If you discover a vulnerability, contact security@solara-inc.in for swift review and remediation.`,
    author: 'Solara Security Team'
  },
  {
    id: 'ann-005',
    title: 'Lightweight Footprint: Less Than 60 MB RAM Idle Overhead',
    date: '2026-09-20',
    category: 'Update',
    summary: 'Engineered with Rust and lightweight Vue for near-zero memory impact while playing.',
    content: `Unlike heavy Electron launchers that consume 500+ MB of RAM, Solara is built with Tauri and native Rust, keeping memory usage minimal so your PC dedicates its power to the game.`,
    author: 'Solara Performance Team'
  },
  {
    id: 'ann-004',
    title: 'ARM64 & x86_64 Architecture Planning: Future Silicon Support',
    date: '2026-09-19',
    category: 'Update',
    summary: 'Our Rust codebase is architected for clean cross-compilation across architectures.',
    content: `While Windows x86_64 is our primary target today, our modular Rust core prepares Solara for future Windows on ARM Snapdragon hardware.`,
    author: 'Solara Architecture Team'
  },
  {
    id: 'ann-003',
    title: 'Quick Instance Duplication: Safe Testing Grounds for Risky Mods',
    date: '2026-09-18',
    category: 'Feature',
    summary: 'Clone an existing instance with all mods and settings intact before testing experimental updates.',
    content: `Want to try 50 new mods without risking your primary survival world? Duplicate your instance in one click to create an instant safe sandbox.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-002',
    title: 'Solara Launcher Open Development: Public Roadmap and Changelogs',
    date: '2026-09-17',
    category: 'Community',
    summary: 'Track our progress transparently with public releases, commit histories, and open verification tests.',
    content: `We believe in honest, transparent software development. Every release is accompanied by exact verification records, checksums, and public changelogs.`,
    author: 'Solara Core Team'
  },
  {
    id: 'ann-001',
    title: 'Welcome to Solara: The Fast, Beautiful Minecraft Launcher',
    date: '2026-09-16',
    category: 'Community',
    summary: 'Welcome to a fresh, independent Minecraft Java experience built for speed, privacy, and control.',
    content: `Thank you for joining us on this journey! Solara Launcher is built for players who want a fast, beautiful, and private way to play Minecraft Java Edition. No bloat, no tracking, just your next world waiting.`,
    author: 'Solara Team',
    tag: 'Welcome'
  }
];

export function getLatestAnnouncements(count: number = 3): LauncherAnnouncement[] {
  return ANNOUNCEMENTS.slice(0, count);
}

export function getAllAnnouncements(limit: number = 50): LauncherAnnouncement[] {
  return ANNOUNCEMENTS.slice(0, limit);
}

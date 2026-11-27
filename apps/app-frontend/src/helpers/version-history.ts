export interface VersionChangelog {
	version: string
	title: string
	date: string
	channel: string
	isLatest?: boolean
	featuresAdded: string[]
	issuesFixed: string[]
	futureUpdates: string[]
}

export const VERIFIED_VERSION_HISTORY: VersionChangelog[] = [
	{
		version: 'V-0.0.105',
		title: 'Restored Beloved Classic Curved-Frame Sidebar & Title Bar Shell',
		date: 'November 27, 2026',
		channel: 'Testing Preview',
		isLatest: true,
		featuresAdded: [
			'Restored the beloved seamless elevated outer L-frame shell, recessed curved pure-OLED inner stage, bouncy circular navigation icons, classic Quick Instance Switcher, and crisp top title bar.',
		],
		issuesFixed: [
			'Preserved all recent Solara upgrades (native Solara Discord Rich Presence, CurseForge ACL fix, YouTube preview fix, strict no-auto-restart updates, custom OLED App & Instance Settings modals, frosted side notifications, and native solara:// shortcuts).',
		],
		futureUpdates: [
			'Wire packages/solara-core modules live into Solara Launcher step-by-step (JRE probing -> Mojang/Fabric/Quilt -> Forge/NeoForge -> direct JVM launch -> SQLite instance store).',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.104',
		title: 'Sleek Floating Solara Glass Dock & Minimal Pure-OLED Top Bar',
		date: 'November 24, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Replaced the wide text sidebar with a sleek compact floating Solara glass dock (72px rail, rounded 22px frosted dock, 42px squircle icons with glowing cyan active pills and micro left glow pip) so your games, modpacks, and skins get maximum screen space.',
			'Upgraded Recent Instances in the floating dock to crisp rounded-squircle icons with live pulsing cyan running dots and full right-click Play/Stop/Open Folder menus.',
			'Simplified the Top Title Bar to a clean pure-OLED minimal bar with Solara Launcher branding, a frosted pill Back/Forward capsule, airy breadcrumbs, dark glass status pill, and flush rounded window controls.',
		],
		issuesFixed: [
			'Restored spacious wide layout across Home, Library, Modpacks, Skins, and Instance pages while keeping a clean futuristic Solara identity distinct from Modrinth.',
		],
		futureUpdates: [
			'Wire packages/solara-core modules live into Solara Launcher step-by-step (JRE probing -> Mojang/Fabric/Quilt -> Forge/NeoForge -> direct JVM launch -> SQLite instance store).',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.103',
		title: 'Custom Solara Labeled Sidebar, Top Command Title Bar & Independent solara-core Foundation',
		date: 'November 24, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Replaced the narrow Modrinth-style icon strip and curved L-frame cutout with a sleek labeled pure-OLED Solara Left Sidebar featuring glowing light-blue active indicators, named Recent Instances pills with live running dots, and a pinned + New Instance & Settings action dock.',
			'Redesigned the Top Title Bar with a 2px Solara cyan-to-gold micro-accent hairline, aligned Solara Launcher crest badge, pill Back/Forward navigation capsule, frosted pill breadcrumbs, borderless status pill, and flush frameless window controls.',
			'Added the independent packages/solara-core Rust launcher core crate (pure-Rust Java probing, Mojang Piston manifest/downloader, Fabric & Quilt loader merger, and direct JVM launch builder with 10/10 passing unit tests) for staged module-by-module engine independence.',
		],
		issuesFixed: [
			'Removed the legacy rounded-top-left L-border chrome and floating corner window controls box so the entire desktop shell looks and feels 100% Solara.',
		],
		futureUpdates: [
			'Wire packages/solara-core modules live into Solara Launcher step-by-step (JRE probing -> Mojang/Fabric/Quilt -> Forge/NeoForge -> direct JVM launch -> SQLite instance store).',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.102',
		title: 'Custom Pure-OLED Solara App Settings, Native solara:// Shortcuts & Major Upstream Cleanup',
		date: 'November 24, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Built a 100% custom pure-#000000 OLED Solara App Settings modal with a branded left navigation sidebar, light-blue active tabs, and borderless raised cards across Java & Memory, Game Window, Launch Hooks, and Troubleshooting.',
			'Added native solara:// deep-link handling in Rust and switched desktop instance shortcuts to solara://launch/instance while keeping external .mrpack compatibility.',
			'Upgraded the Title Bar, Sidebar, Getting Started card, Screenshots toolbar, Project Description/Gallery pages, Export Pack modal, Error modal, and Delete Instance modal to borderless pure-OLED Solara styling.',
		],
		issuesFixed: [
			'Removed over 500 unused legacy Modrinth files (blog, moderation, old library, friends, and cloud-sync modules), dead fallback URLs, and migrated launcher localStorage keys cleanly to solara-* with automatic fallback.',
		],
		futureUpdates: [
			'Independent Solara Rust launcher core (packages/solara-core) replacing inherited Theseus engine module-by-module.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.101',
		title: 'Solara Discord Rich Presence & Fixed CurseForge Downloads',
		date: 'November 24, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Enabled native Solara Discord Rich Presence using the official Solara Launcher Discord app, Solara logo artwork, live game/modloader status, elapsed play timer, background Discord detection, and a "Get Solara Launcher" profile button.',
			'Added a Discord Rich Presence toggle inside Settings → Appearance so you can turn Discord status sharing on or off instantly.',
		],
		issuesFixed: [
			'Fixed the CurseForge ACL permission error ("Command plugin:curseforge|curseforge_install_content not allowed by ACL") so installing CurseForge mods, shaders, resource packs, datapacks, and modpacks works cleanly again.',
			'Fixed YouTube "Video Player Configuration Error" (Error 153) inside project descriptions by switching the launcher referrer policy to strict-origin-when-cross-origin and attaching proper YouTube nocookie iframe referrer attributes.',
			'Hardened update handling so background downloads never queue or trigger an automatic restart — clicking the top-bar update pill opens the changelog modal and Solara only restarts when you explicitly click Update Now.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.100',
		title: 'Solara Frosted OLED Side Notifications, Custom Instance Settings & Unified Light-Blue Accent',
		date: 'November 24, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Redesigned all side notifications and download progress popups into sleek frosted pure-OLED Solara glass cards with a glowing top accent hairline, soft icon badges, and smooth slide animations.',
			'Built a 100% custom pure-#000000 OLED Solara Instance Settings modal with a profile identity sidebar, light-blue active tabs, quick Open Game Folder shortcut, and borderless raised cards across every settings tab.',
		],
		issuesFixed: [
			'Unified green Update, Update All, success notification, and download progress accents into Solara Launcher’s signature light-blue theme color.',
			'Removed the unused Cloud Sync tab and stock Modrinth modal chrome from Instance Settings for a cleaner Solara experience.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.99',
		title: 'Pure-OLED Instance/Library/Browser UI, Classic ASCII Console Logo, Custom Instance Settings & Fixed Updates',
		date: 'November 23, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Redesigned the Instance header, tabs, Mods & Packs workspace, Library grid, and Add Mods browser into a borderless pure-#000000 OLED style with silky raised cards and Modrinth-style long horizontal browse cards.',
			'Customized the Instance Settings menu with a Solara profile identity card, one-click Icon Studio & Upload controls, quick Game Folder footer shortcut, and clean borderless settings cards.',
			'Replaced the console logo with colorful classic / \\ | _ ASCII art spelling SOLARA.',
			'Restored the New Version Available popup with Features Added, Issues Fixed, and Future Updates Implementation, plus added Update Selected in the bulk-selection dock.',
		],
		issuesFixed: [
			'Stopped Solara Launcher from restarting automatically on its own when an update finishes downloading — it now waits until you click Update Now.',
			'Fixed the single-mod Update button and Bulk Update All button inside instance management by passing the correct item IDs and UpdateAllSelection payloads.',
			'Fixed Install buttons overflowing outside narrow mod cards inside Add Mods and removed harsh gray card/toolbar outlines across Instance, Add Mods, and Library pages.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.83',
		title: 'Fixed Modrinth Loading, Cover-Banner Modpack UI, Redesigned Create Instance & Solara Icon Studio',
		date: 'November 22, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Completely redesigned ModPackInstaller with rich cover-banner cards (for both Modrinth and CurseForge), Grid/List view toggle, native Solara dropdown filters, and clean pagination.',
			'Redesigned both steps of Create Instance with side-by-side setup hero cards, unified icon & name header, 5-column visual loader selector, and live configuration summary.',
			'Replaced procedural icons with 30 crisp online Microsoft Fluent Emoji 3D PNG icons (MIT licensed) and built Solara Icon Studio with 3D Icons search/categories, Monogram Badge generator, Framed Upload, dual custom color pickers, gradient angles, texture patterns, HUD/ring/border frames, scale slider, and depth shadow toggle.',
		],
		issuesFixed: [
			'Fixed Modrinth modpacks failing with "Unable to load modpacks right now" by sending valid Labrinth v3 query syntax (?limit=...&new_filters=...) and reading mrpack_loaders, game_versions, and featured_gallery properly.',
			'Replaced native Windows HTML select boxes and checkboxes in ModPackInstaller with Solara-themed Combobox and Checkbox controls for OLED and White modes.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.82',
		title: 'Redesigned ModPackInstaller (Modrinth & CurseForge) & Custom Solara Instance Creator',
		date: 'November 21, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Completely redesigned ModPackInstaller to match Solara Launcher OLED and White themes with a clean top Modrinth | CurseForge source switcher.',
			'Added live CurseForge modpack browsing, version picker modal, and native Rust CurseForge .zip modpack installer powered securely through Solara private server proxy (zero API keys embedded in the app).',
			'Streamlined the Create Instance modal into two clean paths: Browse and download modpacks or Create your custom setup.',
			'Replaced all Modrinth instance icons with 24 original 512x512 3D isometric voxel Solara icons with Randomize, Choose specific, and Upload custom controls.',
		],
		issuesFixed: [
			'Removed legacy browse sidebar and cluttered Create Instance options for a cleaner OLED & White desktop flow.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.81',
		title: 'Clean, Native Desktop Settings UI Without Gradient Clutter',
		date: 'November 20, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Refined the Settings modal, sidebar navigation, Theme selector previews, and Resources tab into a crisp, authentic desktop layout matching Java, Window, and Troubleshooting tabs.',
			'Updated OLED and White theme preview cards with realistic mini window layouts and pure black/white surfaces without radial glow smudges or marketing sub-badges.',
		],
		issuesFixed: [
			'Removed over-decorated gradient icon boxes, two-color active tab gradients, and glowing pill badges from Settings.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.80',
		title: 'Cleaner Solara Settings Footer Without OS Version Clutter',
		date: 'November 20, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Simplified the bottom-left Solara Settings footer badge to display only the clean Solara Launcher version without the Windows OS build line underneath.',
		],
		issuesFixed: [
			'Removed redundant operating system build version text from the Settings sidebar footer.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.79',
		title: 'Redesigned Solara Settings Control Center & Card-Based Resources UI',
		date: 'November 20, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Upgraded the entire Settings modal into a custom Solara Settings experience with a branded Solara crest header, version badge, left-accented active tab indicator, and elevated sidebar footer card in both OLED and White themes.',
			'Removed the unused Syncing tab from Settings for an even cleaner sidebar.',
			'Redesigned Settings → Resources into three clean Solara cards: Content Storage (cyan/violet/amber breakdown bar, stat tiles, cache limit & cleanup), App Directory (monospace click-to-copy path bar & quick actions), and a 2-column Transfer & Disk Limits card with live value chips.',
		],
		issuesFixed: [
			'Improved visual hierarchy and spacing across Settings tabs for both OLED True Black and White Clean Daylight themes.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.78',
		title: 'Custom Solara Theme Previews & Streamlined Settings Display Section',
		date: 'November 20, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Renamed Settings → Appearance to Theme and replaced the generic theme preview boxes with custom mini Solara Launcher window previews for both OLED and White themes.',
			'Removed the Features and Behavior tabs from Settings so Display focuses cleanly on Theme.',
			'Locked clean launcher defaults across frontend and backend: Instance Worlds OFF, Files OFF, Screenshots ON, Sidebar Screenshots OFF, Skin Studio ON, Quick Instances max 20, Jump In OFF, and all Behavior toggles (Minimize app, Refocus on close, Hide right sidebar, Compact mode, Show play time, Hide nametag, Unknown modpack warning, Skip non-essential warnings) permanently OFF.',
		],
		issuesFixed: [
			'Ensured existing profiles automatically apply the locked Feature and Behavior defaults on startup.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.77',
		title: 'Streamlined Settings Menu & Permanently Locked Off Advanced Rendering / System Window Frame',
		date: 'November 20, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Removed the unused Language tab from the Settings menu for a cleaner, more focused settings layout.',
			'Removed Advanced rendering and System window frame toggles from Settings → Appearance and locked both permanently OFF across frontend and Rust backend.',
			'Streamlined Windows preview packaging workflow to avoid duplicate debug test builds alongside the parallel Windows source-check job.',
		],
		issuesFixed: [
			'Ensured existing user profiles with saved advanced rendering or native OS window frame flags are automatically reset to OFF so the custom Solara title bar and fast rendering stay consistent.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.76',
		title: 'ModPack Navigation Fix, Original Solara Downloads UI & Fast Windows CI Builds',
		date: 'November 20, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Redesigned the top Download Manager into an original Solara Downloads capsule and drawer featuring segmented Active / History tabs, cyan-to-gold gradient progress bars, status pills, live speed chip, and borderless task cards in both OLED and White themes.',
			'Rendered project compatibility, links, tags, creators, and details inline in a clean responsive two-column layout on ModPackInstaller project pages.',
			'Switched release builds to thin LTO and upgraded Windows CI checks and builds to Swatinem/rust-cache@v2 for dramatically faster build times.',
		],
		issuesFixed: [
			'Fixed the ModPackInstaller navigation freeze bug where opening a modpack detail page stopped Home, Back, Versions, Gallery, Library, and Skin Studio buttons from updating the UI due to a missing #sidebar-teleport-target mount.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.75',
		title: 'Beautified Home Welcome Hero with Floating Logo & Original Solara Skin Customizer',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Moved the animated Solara logo cleanly above the Home screen Welcome message and upgraded the Welcome typography with a cyan-to-gold gradient username, divider accent, and refined ambient glow.',
			'Redesigned the Customize Skin modal into an original Solara Studio Customizer featuring Classic/Slim model cards, drag-and-drop texture replacement, borderless cape selector, and a live right-side 3D preview stage with pose preview pills.',
			'Optimized Windows CI caching with Rust target prefix fallback and 4 parallel build jobs for faster releases.',
		],
		issuesFixed: [
			'Fixed player 3D emotes not playing after the pose-only refactor by restoring SolaraCharacterActionController.start() and verifying all 6 poses (Wave, Sit, Jump, Stretch, Look, Salute) trigger smoothly.',
			'Replaced hardcoded Steve skin fallback path in the skin customizer with the bundled asset import.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.74',
		title: 'Clean Animated Home Logo, New Solara-Inc Splash Screen, Sit-Stand Fix & Auto-Download Updates',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Simplified the Home screen left emblem to a clean floating animated Solara logo without spinning rings or extra label clutter.',
			'Redesigned the startup loading screen with an animated Solara logo, Solara-Inc title, smooth gradient progress bar, and live startup stage messages.',
			'Added automatic background update downloading with Minisign verification so clicking Update Now installs immediately, and already-downloaded updates auto-install on next launch.',
		],
		issuesFixed: [
			'Fixed the 3D character Sit & Stand pose bug where legs stayed stuck bent in mid-air after rising until the next animation played.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.73',
		title: 'Clean Pose-Only 3D Character Animations & Borderless Skin Studio Cards',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Removed hand-held tool/item prop animations and focused the 3D character rig on 6 smooth pose-only animations: Wave, Sit & Stand, Victory Jump, Stretch, Look Around, and Salute.',
			'Redesigned Skin Studio cards, pills, and containers to eliminate excessive nested borders for a cleaner, more spacious wardrobe gallery in both OLED and White themes.',
		],
		issuesFixed: [
			'Eliminated inner SkinButton borders that previously nested inside Solara Skin Studio cards.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.72',
		title: 'Animated Solara-Inc Hero Logo, Library Tab, 2-Theme Mode & Top Download Manager',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Added a balanced animated Solara logo with orbital glow and Solara-Inc signature on the Home screen opposite the 3D animated character.',
			'Added a dedicated Library page in the sidebar (replacing Screenshots) to search, filter, launch, and manage all Minecraft instances.',
			'Updated sidebar navigation icons with distinct icons for ModPackInstaller, Skin Studio, and Library.',
			'Placed an always-visible Modrinth-style Download Manager right next to Check for updates showing active Java/instance/modpack downloads and completed history.',
			'Simplified Appearance settings to two clean themes: OLED and White.',
		],
		issuesFixed: [
			'Bridged standalone Java runtime and archive progress bars into the top Download Manager so live and completed tasks stay visible in one place.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.71',
		title: 'Custom 3D Character Animations & Interactive Minecraft Props',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Added 7 custom Solara procedural 3D character animations that play automatically on both Home and Skin Studio: Eating a Golden Apple (with golden crumb particles), Diamond Sword flourish, Diamond Pickaxe mining swings, Glowing Torch cave look-around (with rising embers), friendly Waving, Sitting & Standing back up, and Victory Jump.',
			'Added one-click Action / Emote pills on the Solara Skin Studio pedestal (Wave, Golden Apple, Sword, Pickaxe, Torch, Sit, Jump) and click-to-animate character interactions.',
		],
		issuesFixed: [
			'Protected custom 3D prop meshes from skin texture overrides and camera auto-fit bounding shifts.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.70',
		title: 'Original Solara Skin Studio & Sidebar Home Icon',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Redesigned the Skins page into an original Solara Skin Studio with a top Wardrobe & Cape Editor command bar, segmented My Skins / Presets / Capes tabs, drag-and-drop .PNG banner, and dedicated Cape Vault.',
			'Moved the 3D animated character viewer into a sticky elevated right-side pedestal card with Classic/Slim quick switcher, nametag toggle, and live Apply/Reset/Customize dock.',
			'Changed the sidebar Home navigation button and top breadcrumb icon to a clean Home icon.',
			'Includes the centered premium Home hero with edge-docked 3D animated player skin.',
		],
		issuesFixed: [
			'Resolved dropdown stacking overlap on the centered Home hero and streamlined skin/cape preview switching.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.69',
		title: 'Premium Centered Home Hero & Edge-Docked 3D Player Skin',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Centered the Welcome Back header, gradient player name, greeting, and Sign In / Account pill right in the heart of the Home screen.',
			'Docked the live 3D animated player skin cleanly to the far right side, angled subtly toward the welcome hero.',
			'Added refined ambient radial lighting, hover elevation on cards and buttons, and centered account dropdown styling.',
		],
		issuesFixed: [
			'Improved hero spacing so long Minecraft usernames and the right-docked 3D skin remain cleanly separated at all window sizes.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.68',
		title: 'Home Screen Redesign with 3D Animated Skin & Instant Update Checks',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Redesigned full-screen Home hero featuring personalized Welcome Back header, Sign In / Account dropdown pill (switch accounts, add more accounts, logout), and live 3D animated player skin showcase.',
			'Added below-the-fold "All News from Solara" 5-card strip with Read more modals and "See all news" viewer (capped to last 30 items).',
			'Added "All updates of solara launcher" horizontal update strip with "All version history" modal (capped to last 30 releases).',
		],
		issuesFixed: [
			'Dramatically accelerated "Check for updates" from 20-30 seconds down to ~1 second using pooled IPv4-first and dual-stack racing across GitHub Releases CDN and Solara API.',
			'Eliminated duplicate concurrent startup update fetches between Rust background polling and frontend mount.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.67',
		title: 'Startup Black Screen Fix & Bootstrap Reliability',
		date: 'November 19, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Hardened early launcher initialization and update checks for clean cold-start rendering.',
			'Added full GitHub release notes with direct download links, checksums, and source archives.',
		],
		issuesFixed: [
			'Fixed black screen on launch caused by calling useMinecraftAccounts inside root App.vue without passing appEvents.',
			'Fixed early checkUpdates temporal dead zone error before update state listeners initialized.',
			'Guarded early severe error reporting so startup errors cannot crash before the error modal mounts.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.66',
		title: 'Home Dashboard Redesign, ModPackInstaller & Clean Update Modal',
		date: 'November 18, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Redesigned single-screen Home dashboard with personalized welcome message, Minecraft profile switcher, News, and Version History.',
			'Renamed sidebar catalog entry to ModPackInstaller and focused browsing strictly on modpacks.',
			'Replaced top-bar idle indicator with a Check for Updates button and floating Update Available modal without emojis.',
			'Removed the 300px right sidebar for a wider workspace.',
		],
		issuesFixed: [
			'Decoupled Minecraft account state into a shared composable accessible from the Home dashboard.',
			'Streamlined update checking on startup and manual top-bar click.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
		],
	},
	{
		version: 'V-0.0.65',
		title: 'Solara Cyan Identity & Bundled Dependency Licenses',
		date: 'November 15, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Applied signature Solara cyan branding across Dark, Light, OLED, and Retro themes.',
			'Enabled Guest exploration and installation of community modpacks without third-party account requirements.',
			'Bundled all 714 dependency licenses (1,275 license files) directly into the Windows installer payload.',
			'Added automated installer notices validation checks in CI pipeline.',
		],
		issuesFixed: [
			'Fixed NSIS installer license extraction and dependency manifest verification gaps.',
			'Resolved desktop CSP serialization issues for Tauri runtime security.',
			'Removed unsupported remote shared-instance actions with fail-closed modal.',
		],
		futureUpdates: [
			'Independent custom launcher core architecture replacing inherited engine.',
			'Custom Solara Theme Studio integration with personalized accent controls.',
			'Enhanced offline world backups and automatic snapshot recovery.',
		],
	},
	{
		version: 'V-0.0.63',
		title: 'Full Modrinth Desktop Workspace & PKCE OAuth Integration',
		date: 'November 15, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Integrated full desktop launcher workspace with 3D Skin Studio, Instance Manager, and Worlds view.',
			'Preserved Solara Microsoft OAuth 2.0 PKCE loopback authentication with OS Credential Manager keyring.',
			'Integrated Minisign-signed auto-updater with GitHub Releases CDN and top-bar controls.',
			'Enforced Windows 8 MiB stack reserve and Box::pin async Tauri command wrappers.',
		],
		issuesFixed: [
			'Stripped all telemetry, analytics tracking, advertising modules, and commercial billing views.',
			'Collapsed 300x250 ad container placeholder in shell layout.',
			'Redacted tokens and secrets from SQLite cache and panic hooks.',
		],
		futureUpdates: [
			'Streamlined navigation with dedicated ModPackInstaller mode.',
			'Rebuilt home dashboard with instant Minecraft account switcher and changelog viewer.',
		],
	},
	{
		version: 'V-0.0.62',
		title: 'Website Decoupling & Toolchain Modernization',
		date: 'November 15, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Decoupled launcher website UI into dedicated apps/website structure.',
			'Upgraded toolchain to Node 24.15+, pnpm 12.11, and Temurin JDK 17 in CI.',
			'Audited all 825 transitive Windows crate dependencies.',
		],
		issuesFixed: [
			'Resolved unmanifested crate documentation in THIRD_PARTY_NOTICES.',
			'Corrected dependency license checker for pnpm workspace lockfile resolution.',
		],
		futureUpdates: [
			'Full guest catalog exploration for community content.',
			'Streamlined single-screen home dashboard layout.',
		],
	},
	{
		version: 'V-0.0.61',
		title: 'Skin Studio & Mojang Sync Hardening',
		date: 'November 15, 2026',
		channel: 'Testing Preview',
		featuresAdded: [
			'Hardened 3D skin preview rendering and cape texture loading.',
			'Added automatic fallback to default player head avatars when network is offline.',
		],
		issuesFixed: [
			'Fixed skin upload race condition when switching equipped skins rapidly.',
			'Resolved Mojang profile cache invalidation on token refresh.',
		],
		futureUpdates: [
			'Dynamic home dashboard welcoming header.',
			'Direct ModPackInstaller integration.',
		],
	},
]

export function getVersionChangelog(version?: string): VersionChangelog {
	if (!version) return VERIFIED_VERSION_HISTORY[0]
	const clean = version.startsWith('V-') ? version : `V-${version}`
	return (
		VERIFIED_VERSION_HISTORY.find((v) => v.version === clean || v.version.toLowerCase() === version.toLowerCase()) ?? {
			version: clean,
			title: `Solara Launcher ${clean}`,
			date: 'Latest Release',
			channel: 'Testing Preview',
			featuresAdded: [
				'Enhanced Solara Launcher performance and responsiveness.',
				'Instant Minecraft account management and skin preview.',
				'Streamlined modpack installation from ModPackInstaller.',
			],
			issuesFixed: [
				'Automatic update check improvements.',
				'Layout optimization and full-width viewport alignment.',
			],
			futureUpdates: [
				'Continued custom core independence.',
				'Further theme customization options.',
			],
		}
	)
}

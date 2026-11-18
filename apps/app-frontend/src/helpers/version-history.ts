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
		version: 'V-0.0.65',
		title: 'Solara Cyan Identity & Bundled Dependency Licenses',
		date: 'November 15, 2026',
		channel: 'Testing Preview',
		isLatest: true,
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

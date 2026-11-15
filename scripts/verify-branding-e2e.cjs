// Browser/IPC fixtures only. No live account, game, installer, or service acceptance.
const { createRequire } = require('node:module')
const path = require('node:path')
const root = path.resolve(__dirname, '..')
const output = path.join(root, 'checks/rebrand')
const base = process.env.SOLARA_DESKTOP_URL || 'http://127.0.0.1:1420'
const requireProject = createRequire(path.join(root, 'package.json'))
const { chromium } = requireProject('playwright')
const fs = require('node:fs')
const assert = require('node:assert/strict')
const releaseCatalog = JSON.parse(fs.readFileSync(path.join(root, 'builds/releases.json'), 'utf8'))
const newsRelease = releaseCatalog.releases.filter(release => release.state === 'published').at(-1)
;(async () => {
	const browser = await chromium.launch({
		headless: true,
		executablePath: process.env.SOLARA_CHROMIUM_PATH,
		args: ['--no-sandbox'],
	})
	const page = await browser.newPage({ viewport: { width: 1280, height: 800 } })
	const errors = []
	page.on('pageerror', (e) => errors.push(e.message))
	await page.route('**/*', async (route) => {
		const url = new URL(route.request().url())
		if (['127.0.0.1', 'localhost'].includes(url.hostname)) return route.continue()
		return route.fulfill({
			status: 200,
			contentType: 'application/json',
			body: url.pathname.includes('geoip')
				? '"IN"'
				: url.pathname.includes('articles')
					? '{"articles":[]}'
					: url.pathname.includes('bootstrap')
						? JSON.stringify({ releases: [newsRelease] })
						: '[]',
		})
	})
	await page.addInitScript(() => {
		const callbacks = new Map()
		let next = 0
		window.__MOCK_CALLS__ = []
		window.__TAURI_OS_PLUGIN_INTERNALS__ = {
			osType: 'windows',
			platform: 'windows',
			family: 'windows',
			arch: 'x86_64',
			version: '10.0.26100',
		}
		const settings = {
			theme: 'dark',
			locale: 'en-US',
			native_decorations: false,
			advanced_rendering: true,
			toggle_sidebar: false,
			sync_theme_across_devices: false,
			sync_behavior_across_devices: false,
			sync_features_across_devices: false,
			show_files_tab_in_instances: true,
			show_worlds_tab_in_instances: true,
			show_screenshots_tab_in_instances: true,
			show_skin_selector_in_sidebar: true,
			refocus_on_game_close: false,
			compact_instance_cards: false,
			show_play_time: true,
			warn_on_unknown_modpacks: true,
			skip_non_essential_warnings: false,
			show_jump_in: true,
			always_show_copy_details: false,
			hide_installed_modpacks: false,
			advanced_filters_collapsed: false,
			dismissed_photosensitivity_filter_warning: true,
			developer_mode: false,
			feature_flags: {},
			pending_update_toast_for_version: null,
			telemetry: false,
			discord_rpc: false,
			personalized_ads: false,
			custom_dir: null,
			prev_custom_dir: null,
			memory: { min: 1024, max: 4096 },
			game_resolution: [854, 480],
			extra_launch_args: [],
			custom_env_vars: [],
			java_globals: {},
			hooks: {},
			max_concurrent_downloads: 4,
			max_concurrent_writes: 4,
		}
		window.__TAURI_INTERNALS__ = {
			metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
			transformCallback(fn) {
				const id = ++next
				callbacks.set(id, fn)
				return id
			},
			unregisterCallback(id) {
				callbacks.delete(id)
			},
			convertFileSrc(path) {
				return 'asset://localhost/' + encodeURIComponent(path)
			},
			async invoke(cmd, args = {}) {
				window.__MOCK_CALLS__.push(cmd)
				if (cmd === 'plugin:event|listen') return ++next
				if (cmd === 'plugin:app|version') return '0.0.63'
				if (cmd === 'plugin:app|name') return 'Solara Launcher'
				if (cmd === 'plugin:os|type' || cmd === 'plugin:os|platform') return 'windows'
				if (cmd === 'plugin:os|version') return '10.0.26100'
				if (cmd === 'plugin:os|arch') return 'x86_64'
				if (cmd === 'plugin:settings|settings_get') return structuredClone(settings)
				if (cmd === 'plugin:settings|settings_set') {
					Object.assign(settings, args.settings)
					return null
				}
				if (cmd === 'plugin:onboarding-checklist|get_onboarding_checklist')
					return {
						has_created_instance: false,
						has_logged_into_minecraft: false,
						has_logged_into_modrinth: false,
						show_checklist: true,
					}
				if (/global_synced|initialize_synced/.test(cmd))
					return {
						screenshots: false,
						servers: false,
						resource_packs: false,
						shader_packs: false,
						game_options: false,
					}
				if (/progress_bars_list/.test(cmd)) return {}
				if (cmd === 'check_app_update') return { stage: 'development', state: 'development' }
				if (/get_opening_command|get_default_user|is_dev/.test(cmd)) return null
				if (/get_country/.test(cmd)) return 'IN'
				if (cmd === 'plugin:install|install_job_list') return []
				if (cmd === 'plugin:jre|jre_get_max_memory') return 16384
				if (cmd === 'plugin:utils|get_os') return 'Windows'
				if (
					/get_categories|get_loaders|get_game_versions|get_users|get_all|instance_list|list_groups|get_hidden_ids|list_processes|process_list|get_many/.test(
						cmd,
					)
				)
					return []
				if (/get_available_skins|get_available_capes/.test(cmd)) return []
				if (
					cmd.includes('is_maximized') ||
					cmd.includes('is_fullscreen') ||
					cmd.includes('is_focused')
				)
					return false
				if (cmd.includes('scale_factor')) return 1
				if (cmd.includes('inner_size')) return { width: 1280, height: 800 }
				if (cmd.includes('plugin:cache|')) return []
				return null
			},
		}
		window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} }
	})
	try {
		await page.goto(base, { waitUntil: 'networkidle', timeout: 60000 })
		await page.getByRole('heading', { name: 'Welcome to Solara', exact: true }).waitFor()
		await page.getByText(`Solara Launcher ${newsRelease.publicVersion}`, { exact: true }).waitFor()
		await page.waitForTimeout(500)
		fs.mkdirSync(output, { recursive: true })
		assert.equal(await page.title(), 'Solara Launcher')
		assert(!/Modrinth|Theseus/.test(await page.locator('body').innerText()))
		assert.equal(
			await page.getByRole('button', { name: 'Sign in to Minecraft', exact: true }).count(),
			2,
		)
		const themes = []
		for (const theme of ['dark', 'light', 'oled', 'retro']) {
			await page.evaluate(async (theme) => {
				const { useTheme } = await import('/src/composables/use-theme.ts')
				useTheme().preferred = theme
			}, theme)
			await page.waitForTimeout(600)
			themes.push(
				await page.evaluate(() => {
					const css = getComputedStyle(document.documentElement)
					const el = document.querySelector('h1')
					return {
						mode: document.documentElement.className,
						brand: css.getPropertyValue('--color-brand').trim(),
						accentContrast: css.getPropertyValue('--color-accent-contrast').trim(),
						welcome: el.getBoundingClientRect().toJSON(),
						sidebar: document.querySelector('.app-sidebar').getBoundingClientRect().toJSON(),
					}
				}),
			)
			await page.screenshot({ path: path.join(output, `${theme}.png`) })
		}
		await page.evaluate(async () => {
			const { useTheme } = await import('/src/composables/use-theme.ts')
			useTheme().preferred = 'dark'
		})
		await page.locator('.app-grid-navbar button.nav-button').last().click()
		await page.waitForTimeout(700)
		const settingsBody = await page.locator('body').innerText()

		assert(!/Sign in to (?:Modrinth|the catalog)|Sync theme across devices/.test(settingsBody))
		await page.screenshot({ path: path.join(output, 'settings.png') })
		for (const [tab, id] of [
			['Features', 'sync-features-across-devices'],
			['Behavior', 'sync-behavior-across-devices'],
		]) {
			await page.getByRole('button', { name: tab, exact: true }).click()
			await page.waitForTimeout(500)
			assert.equal(await page.locator('#' + id).count(), 0)
			assert(!/Modrinth|Theseus|An error occurred/.test(await page.locator('body').innerText()))
			await page.screenshot({ path: path.join(output, `settings-${tab.toLowerCase()}.png`) })
		}
		await page.keyboard.press('Escape')
		await page.waitForTimeout(400)
		const authorized = await page.evaluate(async () =>
			document
				.querySelector('#app')
				.__vue_app__._instance.setupState.catalogUnavailableModal.show(),
		)
		assert.equal(authorized, false)
		await page.waitForTimeout(600)

		await page.getByText('This online feature is unavailable', { exact: true }).waitFor()
		assert.equal(await page.getByRole('button', { name: /sign in/i }).count(), 2)
		await page.screenshot({ path: path.join(output, 'catalog-unavailable.png') })
		await page.getByRole('button', { name: 'Got it', exact: true }).click()
		await page.waitForTimeout(400)
		await page.getByRole('button', { name: 'Create an instance', exact: true }).click()
		await page.waitForTimeout(600)
		assert(
			!/Modrinth|Theseus|Sign in to the catalog|An error occurred/.test(
				await page.locator('body').innerText(),
			),
		)
		await page.screenshot({ path: path.join(output, 'create-instance.png') })
		await page.keyboard.press('Escape')
		await page.waitForTimeout(300)
		await page.evaluate(async () => {
			const { setLocale } = await import('/src/i18n.config.ts')
			await setLocale('de-DE')
		})
		await page.waitForTimeout(500)
		const german = await page.locator('body').innerText()
		assert(!/Modrinth|Theseus|An error occurred/.test(german))
		assert(german.includes('Solara'))
		await page.screenshot({ path: path.join(output, 'german.png') })
		assert.deepEqual(errors, [])
		assert(
			!(await page.evaluate(() => window.__MOCK_CALLS__)).some((cmd) =>
				/mr-auth.*login|plugin:auth\|login/.test(cmd),
			),
		)
		for (const theme of themes) assert.equal(theme.sidebar.width, 300)
		const result = {
			fixture: true,
			title: await page.title(),
			errors,
			themes,
			settings: true,
			accountFeatureAuthorized: authorized,
			instanceWizardOpened: true,
			releaseNews: true,
			germanLocale: true,
			calls: await page.evaluate(() => [...new Set(window.__MOCK_CALLS__)]),
		}
		fs.writeFileSync(path.join(output, 'results.json'), JSON.stringify(result, null, 2))
		console.log(
			JSON.stringify({
				fixture: true,
				themes: themes.length,
				settings: true,
				accountFeatureAuthorized: authorized,
				instanceWizardOpened: true,
			releaseNews: true,
				germanLocale: true,
				errors,
				output,
			}),
		)
	} finally {
		await browser.close()
	}
})().catch((e) => {
	console.error(e)
	process.exitCode = 1
})

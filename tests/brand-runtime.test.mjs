import { test } from 'node:test'
import assert from 'node:assert/strict'
import {
	readThemePreference,
	themePreferenceKeys,
} from '../apps/app-frontend/src/helpers/theme-preferences.ts'
import { developmentCsp } from '../packages/shared/csp.ts'

function storage(initial = {}) {
	const values = new Map(Object.entries(initial))
	return {
		getItem: (key) => values.get(key) ?? null,
		setItem: (key, value) => values.set(key, value),
		values,
	}
}

test('theme rebrand preserves an existing preference and migrates a valid legacy preference', () => {
	const choices = ['dark', 'light', 'oled', 'retro', 'system']
	for (const theme of choices) {
		const saved = storage({ 'modrinth-theme': theme })
		assert.equal(
			readThemePreference(saved, themePreferenceKeys.theme, 'modrinth-theme', choices),
			theme,
		)
		assert.equal(saved.getItem(themePreferenceKeys.theme), theme)
		assert.equal(saved.getItem('modrinth-theme'), theme)
	}
	const saved = storage({
		[themePreferenceKeys.theme]: 'light',
		'modrinth-theme': 'oled',
	})
	assert.equal(
		readThemePreference(saved, themePreferenceKeys.theme, 'modrinth-theme', choices),
		'light',
	)
})

test('theme migration rejects invalid values and works with unavailable or read-only storage', () => {
	assert.equal(readThemePreference(storage({ old: 'invalid' }), 'new', 'old', ['dark']), null)
	assert.equal(
		readThemePreference(
			{
				getItem() {
					throw Error('blocked')
				},
			},
			'new',
			'old',
			['dark'],
		),
		null,
	)
	const saved = {
		getItem: (key) => (key === 'old' ? 'dark' : null),
		setItem() {
			throw Error('read only')
		},
	}
	assert.equal(readThemePreference(saved, 'new', 'old', ['dark']), 'dark')
	assert.equal(readThemePreference(storage({ old: 'light' }), 'new', 'old', ['dark', 'oled']), null)
})

test('development CSP preserves the native policy and adds only the development websocket', () => {
	const policy =
		"default-src 'self'; connect-src 'self' ipc: https://api.modrinth.com; script-src 'self'; object-src 'none'"
	const actual = developmentCsp(policy, 'ws://localhost:1420')
	assert.equal(
		actual,
		"default-src 'self'; connect-src 'self' ipc: https://api.modrinth.com ws://localhost:1420; script-src 'self'; object-src 'none'",
	)
	assert.equal(developmentCsp(actual, 'ws://localhost:1420'), actual)
	const structured = {
		'default-src': ["'self'"],
		'connect-src': ["'self'", 'ipc:'],
	}
	const before = structuredClone(structured)
	assert.match(
		developmentCsp(structured, 'ws://localhost:1420'),
		/connect-src 'self' ipc: ws:\/\/localhost:1420/,
	)
	assert.deepEqual(structured, before)
	assert.match(
		developmentCsp("default-src 'self'", 'ws://localhost:1420'),
		/connect-src 'self' ws:/,
	)
})

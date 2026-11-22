import axe from '@modrinth/assets/solara/instance-icons/axe.png'
import bowArrow from '@modrinth/assets/solara/instance-icons/bow_arrow.png'
import campfire from '@modrinth/assets/solara/instance-icons/campfire.png'
import castle from '@modrinth/assets/solara/instance-icons/castle.png'
import compass from '@modrinth/assets/solara/instance-icons/compass.png'
import crossedSwords from '@modrinth/assets/solara/instance-icons/crossed_swords.png'
import crown from '@modrinth/assets/solara/instance-icons/crown.png'
import crystalBall from '@modrinth/assets/solara/instance-icons/crystal_ball.png'
import dagger from '@modrinth/assets/solara/instance-icons/dagger.png'
import dragon from '@modrinth/assets/solara/instance-icons/dragon.png'
import flame from '@modrinth/assets/solara/instance-icons/flame.png'
import gamepad from '@modrinth/assets/solara/instance-icons/gamepad.png'
import gear from '@modrinth/assets/solara/instance-icons/gear.png'
import gemStone from '@modrinth/assets/solara/instance-icons/gem_stone.png'
import lightning from '@modrinth/assets/solara/instance-icons/lightning.png'
import magicWand from '@modrinth/assets/solara/instance-icons/magic_wand.png'
import mountain from '@modrinth/assets/solara/instance-icons/mountain.png'
import mushroom from '@modrinth/assets/solara/instance-icons/mushroom.png'
import pickaxe from '@modrinth/assets/solara/instance-icons/pickaxe.png'
import pineTree from '@modrinth/assets/solara/instance-icons/pine_tree.png'
import planet from '@modrinth/assets/solara/instance-icons/planet.png'
import potion from '@modrinth/assets/solara/instance-icons/potion.png'
import robot from '@modrinth/assets/solara/instance-icons/robot.png'
import rocket from '@modrinth/assets/solara/instance-icons/rocket.png'
import shield from '@modrinth/assets/solara/instance-icons/shield.png'
import skull from '@modrinth/assets/solara/instance-icons/skull.png'
import telescope from '@modrinth/assets/solara/instance-icons/telescope.png'
import trophy from '@modrinth/assets/solara/instance-icons/trophy.png'
import volcano from '@modrinth/assets/solara/instance-icons/volcano.png'
import wrench from '@modrinth/assets/solara/instance-icons/wrench.png'

import { defineMessages, type MessageDescriptor } from '#ui/composables/i18n'

const names = defineMessages({
	obsidian: { id: 'instance.icon-editor.background.dark-gray', defaultMessage: 'Obsidian' },
	cyan: { id: 'instance.icon-editor.background.cyan', defaultMessage: 'Solara Cyan' },
	cobalt: { id: 'instance.icon-editor.background.blue', defaultMessage: 'Cobalt' },
	indigo: { id: 'instance.icon-editor.background.purple', defaultMessage: 'Indigo' },
	violet: { id: 'instance.icon-editor.background.lavender', defaultMessage: 'Violet' },
	crimson: { id: 'instance.icon-editor.background.rose', defaultMessage: 'Crimson' },
	ember: { id: 'instance.icon-editor.background.orange', defaultMessage: 'Ember' },
	gold: { id: 'instance.icon-editor.background.yellow', defaultMessage: 'Amber Gold' },
	emerald: { id: 'instance.icon-editor.background.green', defaultMessage: 'Emerald' },
	moss: { id: 'instance.icon-editor.background.lime', defaultMessage: 'Forest' },
	magenta: { id: 'instance.icon-editor.background.pink', defaultMessage: 'Magenta' },
	slate: { id: 'instance.icon-editor.background.gray', defaultMessage: 'Steel Slate' },

	crossedSwords: { id: 'instance.icon-editor.symbol.crossed-swords', defaultMessage: 'Crossed Swords' },
	shield: { id: 'instance.icon-editor.symbol.shield', defaultMessage: 'Shield' },
	pickaxe: { id: 'instance.icon-editor.symbol.pickaxe', defaultMessage: 'Pickaxe' },
	axe: { id: 'instance.icon-editor.symbol.axe', defaultMessage: 'Battle Axe' },
	bowArrow: { id: 'instance.icon-editor.symbol.bow-arrow', defaultMessage: 'Bow & Arrow' },
	dagger: { id: 'instance.icon-editor.symbol.dagger', defaultMessage: 'Dagger' },
	crown: { id: 'instance.icon-editor.symbol.crown', defaultMessage: 'Crown' },
	trophy: { id: 'instance.icon-editor.symbol.trophy', defaultMessage: 'Trophy' },

	gemStone: { id: 'instance.icon-editor.symbol.gem-stone', defaultMessage: 'Diamond Gem' },
	crystalBall: { id: 'instance.icon-editor.symbol.crystal-ball', defaultMessage: 'Crystal Orb' },
	magicWand: { id: 'instance.icon-editor.symbol.magic-wand', defaultMessage: 'Magic Wand' },
	potion: { id: 'instance.icon-editor.symbol.potion', defaultMessage: 'Elixir Vial' },
	dragon: { id: 'instance.icon-editor.symbol.dragon', defaultMessage: 'Dragon' },
	flame: { id: 'instance.icon-editor.symbol.flame', defaultMessage: 'Flame' },
	compass: { id: 'instance.icon-editor.symbol.compass', defaultMessage: 'Compass' },
	castle: { id: 'instance.icon-editor.symbol.castle', defaultMessage: 'Fortress' },

	rocket: { id: 'instance.icon-editor.symbol.rocket', defaultMessage: 'Rocket' },
	gear: { id: 'instance.icon-editor.symbol.gear', defaultMessage: 'Machinery Gear' },
	robot: { id: 'instance.icon-editor.symbol.robot', defaultMessage: 'Automaton' },
	lightning: { id: 'instance.icon-editor.symbol.lightning', defaultMessage: 'High Voltage' },
	planet: { id: 'instance.icon-editor.symbol.planet', defaultMessage: 'Ringed Planet' },
	gamepad: { id: 'instance.icon-editor.symbol.gamepad', defaultMessage: 'Gamepad' },
	wrench: { id: 'instance.icon-editor.symbol.wrench', defaultMessage: 'Engineer Wrench' },
	telescope: { id: 'instance.icon-editor.symbol.telescope', defaultMessage: 'Telescope' },

	pineTree: { id: 'instance.icon-editor.symbol.pine-tree', defaultMessage: 'Taiga Pine' },
	mountain: { id: 'instance.icon-editor.symbol.mountain', defaultMessage: 'Alpine Peak' },
	volcano: { id: 'instance.icon-editor.symbol.volcano', defaultMessage: 'Volcano' },
	mushroom: { id: 'instance.icon-editor.symbol.mushroom', defaultMessage: 'Mushroom' },
	campfire: { id: 'instance.icon-editor.symbol.campfire', defaultMessage: 'Base Camp' },
	skull: { id: 'instance.icon-editor.symbol.skull', defaultMessage: 'Hardcore Skull' },
})

export type SymbolCategory = 'combat' | 'magic' | 'tech' | 'world'

export interface SymbolOption {
	id: string
	name: MessageDescriptor
	label: string
	asset: string
	category: SymbolCategory
	keywords: string[]
	excludeFromRandomization?: boolean
}

export const SYMBOL_CATEGORIES: { id: 'all' | SymbolCategory; label: string }[] = [
	{ id: 'all', label: 'All' },
	{ id: 'combat', label: 'Combat & Gear' },
	{ id: 'magic', label: 'Magic & Fantasy' },
	{ id: 'tech', label: 'Tech & Space' },
	{ id: 'world', label: 'Worlds & Survival' },
]

export const backgroundOptions = [
	{
		id: 'dark_gray',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#0f172a',
			bottom_color: '#1e293b',
		},
		name: names.obsidian,
	},
	{
		id: 'cyan',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#0369a1',
			bottom_color: '#06b6d4',
		},
		name: names.cyan,
	},
	{
		id: 'blue',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#1d4ed8',
			bottom_color: '#3b82f6',
		},
		name: names.cobalt,
	},
	{
		id: 'purple',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#4338ca',
			bottom_color: '#6366f1',
		},
		name: names.indigo,
	},
	{
		id: 'lavender',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#7e22ce',
			bottom_color: '#a855f7',
		},
		name: names.violet,
	},
	{
		id: 'rose',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#be123c',
			bottom_color: '#f43f5e',
		},
		name: names.crimson,
	},
	{
		id: 'orange',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#c2410c',
			bottom_color: '#f97316',
		},
		name: names.ember,
	},
	{
		id: 'yellow',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#b45309',
			bottom_color: '#f59e0b',
		},
		name: names.gold,
	},
	{
		id: 'green',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#047857',
			bottom_color: '#10b981',
		},
		name: names.emerald,
	},
	{
		id: 'lime',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#3f6212',
			bottom_color: '#65a30d',
		},
		name: names.moss,
	},
	{
		id: 'pink',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#be185d',
			bottom_color: '#ec4899',
		},
		name: names.magenta,
	},
	{
		id: 'gray',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#334155',
			bottom_color: '#475569',
		},
		name: names.slate,
	},
] as const

export const symbolOptions = [
	// Combat & Gear
	{ id: 'crossed_swords', name: names.crossedSwords, label: 'Crossed Swords', asset: crossedSwords, category: 'combat', keywords: ['pvp', 'fight', 'weapon', 'sword', 'arena'] },
	{ id: 'shield', name: names.shield, label: 'Shield', asset: shield, category: 'combat', keywords: ['defense', 'guard', 'tank', 'knight'] },
	{ id: 'pickaxe', name: names.pickaxe, label: 'Pickaxe', asset: pickaxe, category: 'combat', keywords: ['mine', 'mining', 'tool', 'ore'] },
	{ id: 'axe', name: names.axe, label: 'Battle Axe', asset: axe, category: 'combat', keywords: ['wood', 'weapon', 'viking', 'chop'] },
	{ id: 'bow_arrow', name: names.bowArrow, label: 'Bow & Arrow', asset: bowArrow, category: 'combat', keywords: ['archer', 'range', 'hunt'] },
	{ id: 'dagger', name: names.dagger, label: 'Dagger', asset: dagger, category: 'combat', keywords: ['blade', 'rogue', 'knife'] },
	{ id: 'crown', name: names.crown, label: 'Crown', asset: crown, category: 'combat', keywords: ['king', 'royal', 'smp', 'leader'] },
	{ id: 'trophy', name: names.trophy, label: 'Trophy', asset: trophy, category: 'combat', keywords: ['win', 'champion', 'speedrun', 'cup'] },

	// Magic & Fantasy
	{ id: 'gem_stone', name: names.gemStone, label: 'Diamond Gem', asset: gemStone, category: 'magic', keywords: ['diamond', 'crystal', 'jewel', 'ore'] },
	{ id: 'crystal_ball', name: names.crystalBall, label: 'Crystal Orb', asset: crystalBall, category: 'magic', keywords: ['arcane', 'mystic', 'orb', 'enchant'] },
	{ id: 'magic_wand', name: names.magicWand, label: 'Magic Wand', asset: magicWand, category: 'magic', keywords: ['spell', 'wizard', 'sparkle'] },
	{ id: 'potion', name: names.potion, label: 'Elixir Vial', asset: potion, category: 'magic', keywords: ['alchemy', 'brew', 'lab', 'chemistry'] },
	{ id: 'dragon', name: names.dragon, label: 'Dragon', asset: dragon, category: 'magic', keywords: ['end', 'boss', 'rpg', 'fantasy'] },
	{ id: 'flame', name: names.flame, label: 'Flame', asset: flame, category: 'magic', keywords: ['nether', 'fire', 'blaze', 'hot'] },
	{ id: 'compass', name: names.compass, label: 'Compass', asset: compass, category: 'magic', keywords: ['explore', 'adventure', 'map', 'travel'] },
	{ id: 'castle', name: names.castle, label: 'Fortress', asset: castle, category: 'magic', keywords: ['kingdom', 'medieval', 'build', 'tower'] },

	// Tech & Space
	{ id: 'rocket', name: names.rocket, label: 'Rocket', asset: rocket, category: 'tech', keywords: ['space', 'fast', 'fps', 'boost', 'launch'] },
	{ id: 'gear', name: names.gear, label: 'Machinery Gear', asset: gear, category: 'tech', keywords: ['create', 'tech', 'automation', 'factory'] },
	{ id: 'robot', name: names.robot, label: 'Automaton', asset: robot, category: 'tech', keywords: ['ai', 'mech', 'bot', 'scifi'] },
	{ id: 'lightning', name: names.lightning, label: 'High Voltage', asset: lightning, category: 'tech', keywords: ['energy', 'power', 'speed', 'performance'] },
	{ id: 'planet', name: names.planet, label: 'Ringed Planet', asset: planet, category: 'tech', keywords: ['galaxy', 'cosmos', 'sky', 'orbit'] },
	{ id: 'gamepad', name: names.gamepad, label: 'Gamepad', asset: gamepad, category: 'tech', keywords: ['play', 'minigame', 'arcade', 'fun'] },
	{ id: 'wrench', name: names.wrench, label: 'Engineer Wrench', asset: wrench, category: 'tech', keywords: ['dev', 'test', 'tinker', 'craft'] },
	{ id: 'telescope', name: names.telescope, label: 'Telescope', asset: telescope, category: 'tech', keywords: ['stars', 'astronomy', 'view', 'shaders'] },

	// Worlds & Survival
	{ id: 'pine_tree', name: names.pineTree, label: 'Taiga Pine', asset: pineTree, category: 'world', keywords: ['forest', 'nature', 'biome', 'survival'] },
	{ id: 'mountain', name: names.mountain, label: 'Alpine Peak', asset: mountain, category: 'world', keywords: ['snow', 'terrain', 'world', 'cliff'] },
	{ id: 'volcano', name: names.volcano, label: 'Volcano', asset: volcano, category: 'world', keywords: ['lava', 'magma', 'island', 'nether'] },
	{ id: 'mushroom', name: names.mushroom, label: 'Mushroom', asset: mushroom, category: 'world', keywords: ['forest', 'cozy', 'cottage', 'mooshroom'] },
	{ id: 'campfire', name: names.campfire, label: 'Base Camp', asset: campfire, category: 'world', keywords: ['tent', 'survival', 'cozy', 'smp'] },
	{ id: 'skull', name: names.skull, label: 'Hardcore Skull', asset: skull, category: 'world', keywords: ['hardcore', 'zombie', 'horror', 'dungeon'] },
] as const satisfies readonly SymbolOption[]

export type BackgroundId = (typeof backgroundOptions)[number]['id']
export type SymbolId = (typeof symbolOptions)[number]['id']

export const RANDOM_CONFIG_BLACKLIST = [
	{ background: 'orange', symbol: 'flame' },
	{ background: 'rose', symbol: 'volcano' },
	{ background: 'green', symbol: 'pine_tree' },
	{ background: 'cyan', symbol: 'gem_stone' },
	{ background: 'yellow', symbol: 'crown' },
	{ background: 'yellow', symbol: 'lightning' },
] satisfies readonly { background: BackgroundId; symbol: SymbolId }[]

export const DEFAULT_BACKGROUND_ID = 'dark_gray' satisfies BackgroundId
export const DEFAULT_SYMBOL_ID = 'gem_stone' satisfies SymbolId

const LEGACY_SYMBOL_FALLBACKS: Record<string, string> = {
	solara_core: gemStone,
	wrenth_rinth: gemStone,
	mr_pack: gemStone,
	sword: crossedSwords,
	diamond_sword: crossedSwords,
	diamond_ore: gemStone,
	emerald_block: gemStone,
	amethyst_crystal: crystalBall,
	golden_crown: crown,
	enchanted_shield: shield,
	enchanting_table: magicWand,
	beacon: lightning,
	chest: trophy,
	ender_chest: trophy,
	ender_portal: planet,
	redstone_lamp: lightning,
	redstone_block: lightning,
	tnt: volcano,
	potion_flask: potion,
	snowy_peak: mountain,
	magma_block: volcano,
	lantern: flame,
	anvil: wrench,
	tech_gear: gear,
	cogwheel: gear,
	grass_block: pineTree,
	netherite_block: shield,
	netherite_pickaxe: pickaxe,
}

export function getIconSymbolAsset(symbol: string): string | undefined {
	return (
		symbolOptions.find((option) => option.id === symbol)?.asset ??
		LEGACY_SYMBOL_FALLBACKS[symbol] ??
		gemStone
	)
}

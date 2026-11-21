import amethystCrystal from '@modrinth/assets/solara/instance-icons/amethyst_crystal.png'
import anvil from '@modrinth/assets/solara/instance-icons/anvil.png'
import beacon from '@modrinth/assets/solara/instance-icons/beacon.png'
import campfire from '@modrinth/assets/solara/instance-icons/campfire.png'
import compass from '@modrinth/assets/solara/instance-icons/compass.png'
import diamondOre from '@modrinth/assets/solara/instance-icons/diamond_ore.png'
import diamondSword from '@modrinth/assets/solara/instance-icons/diamond_sword.png'
import emeraldBlock from '@modrinth/assets/solara/instance-icons/emerald_block.png'
import enchantedShield from '@modrinth/assets/solara/instance-icons/enchanted_shield.png'
import enchantingTable from '@modrinth/assets/solara/instance-icons/enchanting_table.png'
import enderPortal from '@modrinth/assets/solara/instance-icons/ender_portal.png'
import goldenCrown from '@modrinth/assets/solara/instance-icons/golden_crown.png'
import grassBlock from '@modrinth/assets/solara/instance-icons/grass_block.png'
import lantern from '@modrinth/assets/solara/instance-icons/lantern.png'
import magmaBlock from '@modrinth/assets/solara/instance-icons/magma_block.png'
import netheriteBlock from '@modrinth/assets/solara/instance-icons/netherite_block.png'
import netheritePickaxe from '@modrinth/assets/solara/instance-icons/netherite_pickaxe.png'
import potionFlask from '@modrinth/assets/solara/instance-icons/potion_flask.png'
import redstoneLamp from '@modrinth/assets/solara/instance-icons/redstone_lamp.png'
import snowyPeak from '@modrinth/assets/solara/instance-icons/snowy_peak.png'
import solaraCore from '@modrinth/assets/solara/instance-icons/solara_core.png'
import techGear from '@modrinth/assets/solara/instance-icons/tech_gear.png'
import tntCrate from '@modrinth/assets/solara/instance-icons/tnt_crate.png'
import treasureChest from '@modrinth/assets/solara/instance-icons/treasure_chest.png'

import { defineMessages, type MessageDescriptor } from '#ui/composables/i18n'

const names = defineMessages({
	cyan: { id: 'instance.icon-editor.background.cyan', defaultMessage: 'Cyan' },
	blue: { id: 'instance.icon-editor.background.blue', defaultMessage: 'Blue' },
	purple: { id: 'instance.icon-editor.background.purple', defaultMessage: 'Purple' },
	lavender: { id: 'instance.icon-editor.background.lavender', defaultMessage: 'Lavender' },
	rose: { id: 'instance.icon-editor.background.rose', defaultMessage: 'Rose' },
	orange: { id: 'instance.icon-editor.background.orange', defaultMessage: 'Orange' },
	yellow: { id: 'instance.icon-editor.background.yellow', defaultMessage: 'Gold' },
	green: { id: 'instance.icon-editor.background.green', defaultMessage: 'Emerald' },
	lime: { id: 'instance.icon-editor.background.lime', defaultMessage: 'Lime' },
	pink: { id: 'instance.icon-editor.background.pink', defaultMessage: 'Pink' },
	gray: { id: 'instance.icon-editor.background.gray', defaultMessage: 'Slate' },
	darkGray: { id: 'instance.icon-editor.background.dark-gray', defaultMessage: 'Obsidian' },

	solaraCore: { id: 'instance.icon-editor.symbol.solara-core', defaultMessage: 'Solara Core' },
	grassBlock: { id: 'instance.icon-editor.symbol.grass-block', defaultMessage: 'Grass Terrain' },
	diamondSword: { id: 'instance.icon-editor.symbol.sword', defaultMessage: 'Diamond Sword' },
	netheritePickaxe: { id: 'instance.icon-editor.symbol.pickaxe', defaultMessage: 'Netherite Pickaxe' },
	diamondOre: { id: 'instance.icon-editor.symbol.diamond-ore', defaultMessage: 'Diamond Ore' },
	emeraldBlock: { id: 'instance.icon-editor.symbol.emerald-block', defaultMessage: 'Emerald Block' },
	amethystCrystal: { id: 'instance.icon-editor.symbol.amethyst-crystal', defaultMessage: 'Amethyst Cluster' },
	goldenCrown: { id: 'instance.icon-editor.symbol.golden-crown', defaultMessage: 'Golden Crown' },
	enchantedShield: { id: 'instance.icon-editor.symbol.enchanted-shield', defaultMessage: 'Solara Shield' },
	campfire: { id: 'instance.icon-editor.symbol.campfire', defaultMessage: 'Campfire' },
	enchantingTable: { id: 'instance.icon-editor.symbol.enchanting-table', defaultMessage: 'Enchanting Table' },
	beacon: { id: 'instance.icon-editor.symbol.beacon', defaultMessage: 'Beacon' },
	treasureChest: { id: 'instance.icon-editor.symbol.chest', defaultMessage: 'Treasure Chest' },
	enderPortal: { id: 'instance.icon-editor.symbol.ender-portal', defaultMessage: 'Ender Portal' },
	redstoneLamp: { id: 'instance.icon-editor.symbol.redstone-lamp', defaultMessage: 'Redstone Lamp' },
	tntCrate: { id: 'instance.icon-editor.symbol.tnt', defaultMessage: 'TNT Crate' },
	potionFlask: { id: 'instance.icon-editor.symbol.potion-flask', defaultMessage: 'Elixir Flask' },
	compass: { id: 'instance.icon-editor.symbol.compass', defaultMessage: 'Explorer Compass' },
	snowyPeak: { id: 'instance.icon-editor.symbol.snowy-peak', defaultMessage: 'Snowy Peak' },
	magmaBlock: { id: 'instance.icon-editor.symbol.magma-block', defaultMessage: 'Magma Core' },
	lantern: { id: 'instance.icon-editor.symbol.lantern', defaultMessage: 'Iron Lantern' },
	anvil: { id: 'instance.icon-editor.symbol.anvil', defaultMessage: 'Blacksmith Anvil' },
	techGear: { id: 'instance.icon-editor.symbol.cogwheel', defaultMessage: 'Tech Gear' },
	netheriteBlock: { id: 'instance.icon-editor.symbol.netherite-block', defaultMessage: 'Netherite Block' },
})

export interface SymbolOption {
	id: string
	name: MessageDescriptor
	asset: string
	category: 'solara' | 'vanilla'
	excludeFromRandomization?: boolean
}

export const backgroundOptions = [
	{
		id: 'dark_gray',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#111827',
			bottom_color: '#1e293b',
		},
		name: names.darkGray,
	},
	{
		id: 'cyan',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#0284c7',
			bottom_color: '#38bdf8',
		},
		name: names.cyan,
	},
	{
		id: 'blue',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#2563eb',
			bottom_color: '#60a5fa',
		},
		name: names.blue,
	},
	{
		id: 'purple',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#4f46e5',
			bottom_color: '#818cf8',
		},
		name: names.purple,
	},
	{
		id: 'lavender',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#9333ea',
			bottom_color: '#c084fc',
		},
		name: names.lavender,
	},
	{
		id: 'rose',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#e11d48',
			bottom_color: '#fb7185',
		},
		name: names.rose,
	},
	{
		id: 'orange',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#ea580c',
			bottom_color: '#fb923c',
		},
		name: names.orange,
	},
	{
		id: 'yellow',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#d97706',
			bottom_color: '#fbbf24',
		},
		name: names.yellow,
	},
	{
		id: 'green',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#059669',
			bottom_color: '#34d399',
		},
		name: names.green,
	},
	{
		id: 'lime',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#65a30d',
			bottom_color: '#a3e635',
		},
		name: names.lime,
	},
	{
		id: 'pink',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#db2777',
			bottom_color: '#f472b6',
		},
		name: names.pink,
	},
	{
		id: 'gray',
		background: {
			type: 'linear-top-down-gradient',
			top_color: '#334155',
			bottom_color: '#64748b',
		},
		name: names.gray,
	},
] as const

export const symbolOptions = [
	// Solara & Adventure originals
	{ id: 'solara_core', name: names.solaraCore, asset: solaraCore, category: 'solara' },
	{ id: 'enchanted_shield', name: names.enchantedShield, asset: enchantedShield, category: 'solara' },
	{ id: 'golden_crown', name: names.goldenCrown, asset: goldenCrown, category: 'solara' },
	{ id: 'amethyst_crystal', name: names.amethystCrystal, asset: amethystCrystal, category: 'solara' },
	{ id: 'potion_flask', name: names.potionFlask, asset: potionFlask, category: 'solara' },
	{ id: 'compass', name: names.compass, asset: compass, category: 'solara' },
	{ id: 'tech_gear', name: names.techGear, asset: techGear, category: 'solara' },
	{ id: 'ender_portal', name: names.enderPortal, asset: enderPortal, category: 'solara' },
	{ id: 'snowy_peak', name: names.snowyPeak, asset: snowyPeak, category: 'solara' },
	{ id: 'magma_block', name: names.magmaBlock, asset: magmaBlock, category: 'solara' },
	{ id: 'anvil', name: names.anvil, asset: anvil, category: 'solara' },
	{ id: 'netherite_block', name: names.netheriteBlock, asset: netheriteBlock, category: 'solara' },

	// Classic voxel originals
	{ id: 'grass_block', name: names.grassBlock, asset: grassBlock, category: 'vanilla' },
	{ id: 'sword', name: names.diamondSword, asset: diamondSword, category: 'vanilla' },
	{ id: 'pickaxe', name: names.netheritePickaxe, asset: netheritePickaxe, category: 'vanilla' },
	{ id: 'diamond_ore', name: names.diamondOre, asset: diamondOre, category: 'vanilla' },
	{ id: 'emerald_block', name: names.emeraldBlock, asset: emeraldBlock, category: 'vanilla' },
	{ id: 'chest', name: names.treasureChest, asset: treasureChest, category: 'vanilla' },
	{ id: 'campfire', name: names.campfire, asset: campfire, category: 'vanilla' },
	{ id: 'enchanting_table', name: names.enchantingTable, asset: enchantingTable, category: 'vanilla' },
	{ id: 'beacon', name: names.beacon, asset: beacon, category: 'vanilla' },
	{ id: 'redstone_lamp', name: names.redstoneLamp, asset: redstoneLamp, category: 'vanilla' },
	{ id: 'lantern', name: names.lantern, asset: lantern, category: 'vanilla' },
	{ id: 'tnt', name: names.tntCrate, asset: tntCrate, category: 'vanilla' },
] as const satisfies readonly SymbolOption[]

export type BackgroundId = (typeof backgroundOptions)[number]['id']
export type SymbolId = (typeof symbolOptions)[number]['id']

export const RANDOM_CONFIG_BLACKLIST = [
	{ background: 'rose', symbol: 'tnt' },
	{ background: 'green', symbol: 'emerald_block' },
	{ background: 'lime', symbol: 'grass_block' },
	{ background: 'yellow', symbol: 'golden_crown' },
	{ background: 'orange', symbol: 'campfire' },
	{ background: 'orange', symbol: 'tech_gear' },
	{ background: 'lavender', symbol: 'amethyst_crystal' },
	{ background: 'cyan', symbol: 'beacon' },
	{ background: 'dark_gray', symbol: 'netherite_block' },
] satisfies readonly { background: BackgroundId; symbol: SymbolId }[]

export const DEFAULT_BACKGROUND_ID = 'dark_gray' satisfies BackgroundId
export const DEFAULT_SYMBOL_ID = 'solara_core' satisfies SymbolId

const LEGACY_SYMBOL_FALLBACKS: Record<string, string> = {
	wrenth_rinth: solaraCore,
	mr_pack: solaraCore,
	cogwheel: techGear,
	redstone_block: redstoneLamp,
	ender_chest: treasureChest,
}

export function getIconSymbolAsset(symbol: string): string | undefined {
	return (
		symbolOptions.find((option) => option.id === symbol)?.asset ??
		LEGACY_SYMBOL_FALLBACKS[symbol] ??
		solaraCore
	)
}

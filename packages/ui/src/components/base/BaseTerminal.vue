<template>
	<div
		class="flex w-full flex-col bg-bg-raised overflow-hidden rounded-2xl"
		:class="minLogHeight ? 'min-h-min' : 'h-full min-h-0'"
	>
		<div
			ref="wrapperRef"
			class="relative mb-2 mt-1 min-h-0 flex-1 overflow-hidden"
			:style="{ minHeight: minLogHeight }"
		>
			<div ref="containerRef" class="absolute inset-0" />
			<Transition name="terminal-loading-fade">
				<div
					v-if="loading"
					class="pointer-events-none absolute inset-0 z-20 animate-bpulse bg-surface-3"
					aria-hidden="true"
				/>
			</Transition>
			<div v-if="!isAtBottom" class="absolute bottom-4 right-4 z-10">
				<IconButton size="xl" label="Scroll to bottom" class="!shadow-2xl" @click="scrollToBottom">
					<ChevronDownIcon />
				</IconButton>
			</div>
		</div>
		<div
			v-if="showInput"
			ref="inputRef"
			class="shrink-0 border-t border-solid border-b-0 border-x-0 border-surface-4 bg-surface-3 p-4"
		>
			<Input
				v-model="commandInput"
				v-tooltip="disableInput ? disableInputTooltip : undefined"
				:icon="TerminalSquareIcon"
				:placeholder="disableInput ? disabledInputPlaceholder : 'Send a command'"
				:disabled="disableInput"
				wrapper-class="w-full"
				size="medium"
				@keydown.enter="submitCommand"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, TerminalSquareIcon } from '@modrinth/assets'
import type { Terminal } from '@xterm/xterm'
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import Input from '#ui/components/base/inputs/Input.vue'
import { useTerminal } from '#ui/composables/terminal'

const props = withDefaults(
	defineProps<{
		scrollback?: number
		minLogHeight?: string
		showInput?: boolean
		disableInput?: boolean
		disableInputTooltip?: string
		disabledInputPlaceholder?: string
		fullscreen?: boolean
		emptyStateType?: 'server' | 'instance'
		loading?: boolean
	}>(),
	{
		scrollback: Infinity,
		minLogHeight: undefined,
		showInput: false,
		disableInput: false,
		disableInputTooltip: undefined,
		disabledInputPlaceholder: 'Server is not running',
		fullscreen: false,
		emptyStateType: undefined,
		loading: false,
	},
)

const SOLARA_LOGO: string[] = [
	'  \x1B[1;38;2;61;214;245m  _____ \x1B[38;2;86;224;250m ____ \x1B[38;2;120;236;214m _      \x1B[38;2;190;240;155m        \x1B[38;2;255;209;102m _____  \x1B[38;2;255;160;72m        \x1B[0m',
	'  \x1B[1;38;2;61;214;245m / ____|\x1B[38;2;86;224;250m/ __ \\\x1B[38;2;120;236;214m| |     \x1B[38;2;190;240;155m   /\\   \x1B[38;2;255;209;102m|  __ \\ \x1B[38;2;255;160;72m   /\\   \x1B[0m',
	'  \x1B[1;38;2;61;214;245m| (___ \x1B[38;2;86;224;250m| |  | |\x1B[38;2;120;236;214m |     \x1B[38;2;190;240;155m  /  \\  \x1B[38;2;255;209;102m| |__) |\x1B[38;2;255;160;72m  /  \\  \x1B[0m',
	'  \x1B[1;38;2;61;214;245m \\___ \\\x1B[38;2;86;224;250m| |  | |\x1B[38;2;120;236;214m |     \x1B[38;2;190;240;155m / /\\ \\ \x1B[38;2;255;209;102m|  _  / \x1B[38;2;255;160;72m / /\\ \\ \x1B[0m',
	'  \x1B[1;38;2;61;214;245m ____) |\x1B[38;2;86;224;250m |__| |\x1B[38;2;120;236;214m |____ \x1B[38;2;190;240;155m/ ____ \\\x1B[38;2;255;209;102m| | \\ \\ \x1B[38;2;255;160;72m/ ____ \\\x1B[0m',
	'  \x1B[1;38;2;61;214;245m|_____/ \x1B[38;2;86;224;250m\\____/\x1B[38;2;120;236;214m|______|\x1B[38;2;190;240;155m_/    \\_\\\x1B[38;2;255;209;102m_|  \\_\\\x1B[38;2;255;160;72m_/    \\_\\\x1B[0m',
]

const EMPTY_STATE_INFO: Record<string, string[]> = {
	server: [
		'  \x1B[90m--------------------------------------------------------\x1B[0m',
		'  \x1B[1;38;2;61;214;245mSOLARA\x1B[1;38;2;255;181;90m LAUNCHER\x1B[0m \x1B[90m| SERVER CONSOLE\x1B[0m',
		'  \x1B[38;2;61;214;245mStatus\x1B[0m  : Waiting for server startup',
		'  \x1B[38;2;255;181;90mAction\x1B[0m  : Press \x1B[1;97mStart\x1B[0m to launch your server and stream logs.',
	],
	instance: [
		'  \x1B[90m--------------------------------------------------------\x1B[0m',
		'  \x1B[1;38;2;61;214;245mSOLARA\x1B[1;38;2;255;181;90m LAUNCHER\x1B[0m \x1B[90m| LIVE GAME CONSOLE\x1B[0m',
		'  \x1B[38;2;61;214;245mStatus\x1B[0m  : No active Minecraft session',
		'  \x1B[38;2;255;181;90mAction\x1B[0m  : Click \x1B[1;38;2;61;214;245mPlay\x1B[0m in the header to stream live game logs.',
		'  \x1B[90mTip: Use the log selector above to inspect past sessions.\x1B[0m',
	],
}

const emit = defineEmits<{
	command: [command: string]
	ready: [terminal: Terminal]
}>()

const containerRef = ref<HTMLElement | null>(null)
const wrapperRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLElement | null>(null)
const commandInput = ref('')

const snappedHeight = ref<number | null>(null)

const showingEmptyState = ref(false)

const {
	terminal,
	searchAddon,
	isAtBottom,
	write,
	writeln,
	clear,
	reset,
	fit: rawFit,
	scrollToBottom,
} = useTerminal({
	container: containerRef,
	scrollback: props.scrollback,
	onReady: (term) => {
		nextTick(() => {
			snapToRows()
		})
		emit('ready', term)
	},
})

function writeEmptyState() {
	if (!terminal.value || !props.emptyStateType) return
	terminal.value.reset()
	const info = EMPTY_STATE_INFO[props.emptyStateType]
	if (info) {
		terminal.value.writeln('')
		for (const line of SOLARA_LOGO) {
			terminal.value.writeln(line)
		}
		terminal.value.writeln('')
		for (const line of info) {
			terminal.value.writeln(line)
		}
	}
	showingEmptyState.value = true
}

function clearEmptyState() {
	if (!showingEmptyState.value) return
	terminal.value?.reset()
	showingEmptyState.value = false
}

function getWrapperMargins() {
	if (!wrapperRef.value) return 0
	const style = getComputedStyle(wrapperRef.value)
	return parseFloat(style.marginTop) + parseFloat(style.marginBottom)
}

function snapToRows() {
	if (!props.fullscreen) {
		snappedHeight.value = null
		return
	}
	const screen = containerRef.value?.querySelector('.xterm-screen') as HTMLElement | null
	if (!screen) {
		snappedHeight.value = null
		return
	}
	const inputH = inputRef.value?.offsetHeight ?? 0
	const borderW = 2
	snappedHeight.value = screen.offsetHeight + getWrapperMargins() + inputH + borderW
}

let resizeDebounce: ReturnType<typeof setTimeout> | null = null

function handleWindowResize() {
	if (!props.fullscreen) return
	if (resizeDebounce) clearTimeout(resizeDebounce)
	snappedHeight.value = null
	resizeDebounce = setTimeout(() => {
		rawFit()
		nextTick(() => snapToRows())
	}, 50)
}

function handleDocumentPointerDown(event: PointerEvent) {
	if (!terminal.value?.hasSelection()) return
	const target = event.target as Node | null
	if (target && containerRef.value?.contains(target)) return
	terminal.value.clearSelection()
}

function handleDocumentKeyDown(event: KeyboardEvent) {
	if (!event.metaKey || event.key.toLowerCase() !== 'a') return
	const target = event.target as Node | null
	const active = document.activeElement
	if (
		!(target && containerRef.value?.contains(target)) &&
		!(active && containerRef.value?.contains(active))
	) {
		return
	}

	event.preventDefault()
	terminal.value?.selectAll()
}

onMounted(() => {
	window.addEventListener('resize', handleWindowResize)
	document.addEventListener('pointerdown', handleDocumentPointerDown)
	document.addEventListener('keydown', handleDocumentKeyDown, true)
})

onBeforeUnmount(() => {
	window.removeEventListener('resize', handleWindowResize)
	document.removeEventListener('pointerdown', handleDocumentPointerDown)
	document.removeEventListener('keydown', handleDocumentKeyDown, true)
	if (resizeDebounce) clearTimeout(resizeDebounce)
})

function fit() {
	rawFit()
	snapToRows()
}

watch(
	() => props.fullscreen,
	() => {
		if (props.fullscreen) {
			nextTick(() => {
				rawFit()
				nextTick(() => snapToRows())
			})
		} else {
			snappedHeight.value = null
		}
	},
)

const submitCommand = () => {
	if (props.disableInput) return
	const cmd = commandInput.value.trim()
	if (!cmd) return
	emit('command', cmd)
	commandInput.value = ''
}

defineExpose({
	write,
	writeln,
	clear,
	reset,
	fit,
	scrollToBottom,
	terminal,
	searchAddon,
	isAtBottom,
	commandInput,
	showingEmptyState,
	writeEmptyState,
	clearEmptyState,
})
</script>

<style>
@keyframes bpulse {
	50% {
		filter: brightness(75%);
	}
}
.animate-bpulse {
	animation: bpulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.xterm {
	height: 100% !important;
}

.xterm-viewport {
	background-color: var(--surface-2) !important;
}

.xterm .xterm-screen {
	width: 100%;
	margin-left: 8px;
	margin-right: auto;
}

.xterm .xterm-rows {
	position: relative;
	z-index: 7;
}

.xterm .xterm-decoration-container {
	overflow: visible !important;
}

.xterm .xterm-decoration-container > div {
	box-sizing: content-box !important;
	margin-left: -12px !important;
	padding-left: 12px !important;
	padding-right: 12px !important;
}

.xterm-scrollable-element > .scrollbar.vertical {
	width: 8px !important;
}

.xterm-scrollable-element > .scrollbar.vertical > div {
	width: 6px !important;
	border-radius: 8px !important;
	contain: layout style !important;
}

.terminal-loading-fade-enter-active,
.terminal-loading-fade-leave-active {
	transition: opacity 250ms ease-in-out;
}

.terminal-loading-fade-enter-from,
.terminal-loading-fade-leave-to {
	opacity: 0;
}
</style>

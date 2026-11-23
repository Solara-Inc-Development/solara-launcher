<template>
	<div
		class="flex w-full flex-col bg-surface-2 overflow-hidden rounded-[20px] border border-solid border-surface-4"
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
	'     \x1B[38;2;32;77;107m▄\x1B[38;2;23;55;78m▄\x1B[38;2;33;81;113;48;2;16;41;59m▀\x1B[38;2;31;77;106;48;2;14;36;53m▀\x1B[38;2;31;77;105;48;2;14;36;54m▀▀▀▀▀▀▀▀▀▀▀▀\x1B[38;2;31;77;106;48;2;14;36;53m▀\x1B[38;2;35;81;113;48;2;16;41;59m▀\x1B[0;38;2;23;55;78m▄\x1B[38;2;32;77;107m▄\x1B[0m   \x1B[0m',
	'   \x1B[38;2;31;74;103m▄\x1B[38;2;29;71;99;48;2;13;34;52m▀\x1B[38;2;13;35;51;48;2;3;13;23m▀\x1B[38;2;3;14;24;48;2;6;20;33m▀\x1B[38;2;4;15;25;48;2;6;20;32m▀\x1B[38;2;4;15;26m▀▀▀\x1B[48;2;7;21;33m▀\x1B[48;2;8;23;35m▀\x1B[48;2;7;21;33m▀\x1B[38;2;5;16;26;48;2;6;19;31m▀▀\x1B[38;2;4;15;26;48;2;7;21;33m▀\x1B[48;2;8;23;34m▀\x1B[48;2;7;21;33m▀\x1B[48;2;6;20;32m▀▀▀\x1B[38;2;4;15;25m▀\x1B[38;2;3;14;24;48;2;6;20;33m▀\x1B[38;2;13;35;51;48;2;3;13;23m▀\x1B[38;2;29;71;99;48;2;13;34;52m▀\x1B[0;38;2;31;74;103m▄\x1B[0m \x1B[0m',
	'  \x1B[38;2;34;81;111m▄\x1B[38;2;22;55;78;48;2;15;40;59m▀\x1B[38;2;4;13;24;48;2;4;15;26m▀\x1B[38;2;6;21;33;48;2;6;20;32m▀\x1B[38;2;6;19;31;48;2;6;19;31m▀▀\x1B[48;2;6;20;32m▀\x1B[48;2;6;19;31m▀\x1B[38;2;7;22;34;48;2;1;6;18m▀\x1B[38;2;4;13;25;48;2;16;45;57m▀\x1B[38;2;0;5;17;48;2;47;115;128m▀\x1B[38;2;3;12;25;48;2;73;162;172m▀\x1B[38;2;7;22;34;48;2;89;182;189m▀\x1B[48;2;96;181;184m▀\x1B[38;2;2;13;25;48;2;89;159;159m▀\x1B[38;2;0;6;19;48;2;66;112;114m▀\x1B[38;2;2;13;26;48;2;23;44;52m▀\x1B[38;2;8;22;33;48;2;0;7;21m▀\x1B[38;2;6;19;31;48;2;6;19;31m▀\x1B[48;2;7;20;32m▀\x1B[48;2;6;19;31m▀▀\x1B[38;2;6;21;33;48;2;6;20;32m▀\x1B[38;2;4;13;24;48;2;4;15;26m▀\x1B[38;2;22;55;78;48;2;15;41;59m▀\x1B[0;38;2;34;81;111m▄\x1B[0m',
	'  \x1B[38;2;31;77;106;48;2;31;77;105m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;6;20;32;48;2;6;20;32m▀\x1B[38;2;6;19;31;48;2;6;19;31m▀\x1B[38;2;6;20;32;48;2;6;19;32m▀\x1B[38;2;5;18;30;48;2;4;13;25m▀\x1B[38;2;3;13;25;48;2;54;143;159m▀\x1B[38;2;44;114;129;48;2;96;230;245m▀\x1B[38;2;86;208;222;48;2;99;215;224m▀\x1B[38;2;107;234;244;48;2;68;141;149m▀\x1B[38;2;112;227;231;48;2;36;74;83m▀\x1B[38;2;112;211;212;48;2;23;47;57m▀\x1B[38;2;119;210;206;48;2;25;48;57m▀\x1B[38;2;135;222;212;48;2;43;73;78m▀\x1B[38;2;146;227;212;48;2;91;137;131m▀\x1B[38;2;135;199;183;48;2;149;206;184m▀\x1B[38;2;76;109;105;48;2;166;218;189m▀\x1B[38;2;3;14;27;48;2;104;135;119m▀\x1B[38;2;6;18;30;48;2;3;14;27m▀\x1B[38;2;7;20;32;48;2;6;20;31m▀\x1B[38;2;6;19;31;48;2;6;19;31m▀\x1B[38;2;6;20;32;48;2;6;20;32m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;31;77;106;48;2;31;77;105m▀\x1B[0m',
	'  \x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;6;20;32;48;2;7;21;33m▀\x1B[38;2;7;22;34;48;2;4;13;25m▀\x1B[38;2;1;6;18;48;2;16;45;58m▀\x1B[38;2;44;114;129;48;2;87;208;222m▀\x1B[38;2;96;230;245;48;2;99;215;224m▀\x1B[38;2;91;199;209;48;2;33;74;85m▀\x1B[38;2;34;74;84;48;2;0;1;14m▀\x1B[38;2;0;5;18;48;2;8;23;35m▀\x1B[38;2;0;8;20;48;2;9;25;37m▀\x1B[38;2;4;15;27;48;2;1;11;24m▀\x1B[38;2;0;9;22;48;2;19;40;50m▀\x1B[38;2;0;6;20;48;2;17;37;46m▀\x1B[48;2;7;20;31m▀\x1B[38;2;49;71;73;48;2;0;3;19m▀\x1B[38;2;151;189;161;48;2;53;71;69m▀\x1B[38;2;182;216;176;48;2;178;201;161m▀\x1B[38;2;89;107;94;48;2;178;192;150m▀\x1B[38;2;0;7;22;48;2;31;42;46m▀\x1B[38;2;8;22;33;48;2;2;14;27m▀\x1B[38;2;6;20;32;48;2;8;21;33m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[0m',
	'  \x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;8;23;35;48;2;7;21;33m▀\x1B[38;2;0;5;17;48;2;3;13;25m▀\x1B[38;2;47;115;128;48;2;73;162;172m▀\x1B[38;2;107;234;244;48;2;112;226;231m▀\x1B[38;2;68;141;149;48;2;36;74;83m▀\x1B[38;2;0;5;18;48;2;0;8;21m▀\x1B[38;2;8;23;35;48;2;10;25;37m▀\x1B[38;2;8;23;34;48;2;0;6;20m▀\x1B[38;2;0;6;20;48;2;27;52;61m▀\x1B[38;2;28;54;62;48;2;132;204;191m▀\x1B[38;2;91;145;141;48;2;55;89;90m▀\x1B[38;2;11;23;34;48;2;0;0;9m▀\x1B[38;2;8;21;32;48;2;0;8;25m▀\x1B[38;2;12;25;36;48;2;0;12;26m▀\x1B[38;2;0;6;22;48;2;0;9;25m▀\x1B[38;2;120;133;109;48;2;60;70;64m▀\x1B[38;2;210;216;161;48;2;212;209;151m▀\x1B[38;2;101;106;86;48;2;153;148;109m▀\x1B[38;2;0;6;23;48;2;2;13;27m▀\x1B[38;2;9;22;34;48;2;7;21;33m▀\x1B[38;2;4;15;26;48;2;5;15;26m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[0m',
	'  \x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;5;16;26;48;2;5;16;26m▀\x1B[38;2;6;18;30;48;2;6;18;30m▀\x1B[38;2;7;22;34;48;2;7;22;34m▀\x1B[38;2;90;183;189;48;2;95;181;184m▀\x1B[38;2;112;211;212;48;2;119;210;206m▀\x1B[38;2;23;47;57;48;2;25;49;57m▀\x1B[38;2;3;14;27;48;2;1;7;20m▀\x1B[38;2;0;5;19;48;2;48;86;91m▀\x1B[38;2;34;64;72;48;2;135;211;198m▀\x1B[38;2;126;199;188;48;2;149;210;189m▀\x1B[38;2;143;204;184;48;2;155;200;171m▀\x1B[38;2;77;103;96;48;2;169;203;168m▀\x1B[38;2;66;85;79;48;2;196;215;167m▀\x1B[38;2;85;98;84;48;2;187;192;143m▀\x1B[38;2;72;80;70;48;2;56;62;58m▀\x1B[38;2;2;12;26;48;2;0;8;24m▀\x1B[38;2;35;45;47;48;2;38;46;47m▀\x1B[38;2;206;196;137;48;2;214;194;131m▀\x1B[38;2;181;167;115;48;2;187;166;111m▀\x1B[38;2;9;22;32;48;2;9;22;32m▀\x1B[38;2;5;18;31;48;2;5;18;31m▀\x1B[38;2;5;16;26;48;2;5;16;26m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[0m',
	'  \x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;7;21;33;48;2;8;23;34m▀\x1B[38;2;2;13;25;48;2;0;6;19m▀\x1B[38;2;89;159;159;48;2;66;112;114m▀\x1B[38;2;135;222;213;48;2;146;227;212m▀\x1B[38;2;42;73;78;48;2;91;137;131m▀\x1B[38;2;1;8;22;48;2;0;6;20m▀\x1B[38;2;36;65;72;48;2;3;14;26m▀\x1B[38;2;39;65;70;48;2;2;13;25m▀\x1B[38;2;35;54;59;48;2;0;0;14m▀\x1B[38;2;60;79;76;48;2;95;106;89m▀\x1B[38;2;186;202;157;48;2;170;175;133m▀\x1B[38;2;165;169;128;48;2;21;28;36m▀\x1B[38;2;28;36;41;48;2;0;12;26m▀\x1B[38;2;0;9;25;48;2;13;26;35m▀\x1B[38;2;1;11;27;48;2;0;6;24m▀\x1B[38;2;66;69;59;48;2;143;129;91m▀\x1B[38;2;236;205;132;48;2;250;209;128m▀\x1B[38;2;169;146;96;48;2;119;103;72m▀\x1B[38;2;1;13;28;48;2;0;7;25m▀\x1B[38;2;7;21;32;48;2;10;22;33m▀\x1B[38;2;5;15;26;48;2;4;15;26m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[0m',
	'  \x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;7;21;33;48;2;6;20;32m▀\x1B[38;2;2;13;26;48;2;8;22;33m▀\x1B[38;2;23;44;52;48;2;0;7;21m▀\x1B[38;2;136;199;183;48;2;75;109;104m▀\x1B[38;2;149;206;184;48;2;167;218;189m▀\x1B[38;2;49;72;73;48;2;151;189;161m▀\x1B[38;2;0;4;20;48;2;55;71;70m▀\x1B[38;2;8;23;34;48;2;0;4;21m▀\x1B[38;2;20;31;39;48;2;6;19;31m▀\x1B[38;2;120;127;101m▀\x1B[38;2;22;32;39;48;2;0;8;24m▀\x1B[38;2;3;14;28;48;2;0;12;27m▀\x1B[38;2;13;26;35;48;2;0;6;24m▀\x1B[38;2;0;3;22;48;2;70;68;57m▀\x1B[38;2;68;68;57;48;2;211;179;113m▀\x1B[38;2;228;193;120;48;2;253;204;120m▀\x1B[38;2;227;184;110;48;2;120;101;69m▀\x1B[38;2;38;41;41;48;2;0;8;26m▀\x1B[38;2;2;14;29;48;2;9;22;32m▀\x1B[38;2;8;21;32;48;2;6;20;32m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;31;77;105;48;2;31;77;105m▀\x1B[0m',
	'  \x1B[38;2;31;77;105;48;2;31;77;106m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;6;20;32;48;2;6;20;32m▀\x1B[38;2;6;19;31;48;2;6;19;31m▀\x1B[38;2;7;20;31;48;2;7;20;32m▀\x1B[38;2;4;14;27;48;2;6;18;30m▀\x1B[38;2;105;134;119;48;2;4;14;27m▀\x1B[38;2;182;216;176;48;2;89;107;94m▀\x1B[38;2;178;201;161;48;2;178;192;149m▀\x1B[38;2;121;134;109;48;2;210;216;160m▀\x1B[38;2;59;68;63;48;2;212;210;151m▀\x1B[38;2;34;44;46;48;2;207;196;137m▀\x1B[38;2;39;46;47;48;2;213;194;131m▀\x1B[38;2;68;69;59;48;2;235;205;132m▀\x1B[38;2;143;129;90;48;2;250;209;128m▀\x1B[38;2;228;193;120;48;2;227;184;110m▀\x1B[38;2;253;204;120;48;2;122;102;69m▀\x1B[38;2;155;126;80;48;2;3;14;29m▀\x1B[38;2;4;14;28;48;2;6;18;30m▀\x1B[38;2;7;19;31;48;2;7;20;31m▀\x1B[38;2;6;19;31;48;2;6;19;31m▀\x1B[38;2;6;20;32;48;2;6;20;32m▀\x1B[38;2;4;15;26;48;2;4;15;26m▀\x1B[38;2;14;36;54;48;2;14;36;54m▀\x1B[38;2;31;77;105;48;2;31;77;106m▀\x1B[0m',
	'  \x1B[38;2;34;81;111m▀\x1B[38;2;15;41;59;48;2;22;56;78m▀\x1B[38;2;4;15;26;48;2;4;13;24m▀\x1B[38;2;6;20;32;48;2;6;21;33m▀\x1B[38;2;6;19;31;48;2;6;19;31m▀▀\x1B[38;2;7;20;32m▀\x1B[38;2;6;19;31m▀\x1B[38;2;0;7;22;48;2;8;22;33m▀\x1B[38;2;31;42;47;48;2;0;14;27m▀\x1B[38;2;101;106;86;48;2;0;6;23m▀\x1B[38;2;153;148;109;48;2;0;13;27m▀\x1B[38;2;181;167;115;48;2;10;22;32m▀\x1B[38;2;187;166;110m▀\x1B[38;2;169;146;96;48;2;0;13;28m▀\x1B[38;2;119;103;72;48;2;0;7;25m▀\x1B[38;2;38;41;41;48;2;0;14;29m▀\x1B[38;2;0;8;26;48;2;9;21;32m▀\x1B[38;2;6;19;31;48;2;6;19;31m▀\x1B[38;2;7;20;31m▀\x1B[38;2;6;19;31m▀▀\x1B[38;2;6;20;32;48;2;6;21;33m▀\x1B[38;2;4;15;26;48;2;4;13;24m▀\x1B[38;2;15;41;59;48;2;22;56;78m▀\x1B[0;38;2;34;80;111m▀\x1B[0m',
	'   \x1B[38;2;31;74;103m▀\x1B[38;2;13;34;52;48;2;29;71;99m▀\x1B[38;2;3;13;23;48;2;13;35;52m▀\x1B[38;2;6;20;33;48;2;3;14;24m▀\x1B[38;2;6;20;32;48;2;4;15;25m▀\x1B[48;2;4;15;26m▀▀▀\x1B[38;2;8;21;33m▀\x1B[38;2;9;22;34m▀\x1B[38;2;7;21;33;48;2;5;15;26m▀\x1B[38;2;5;19;31;48;2;5;16;26m▀▀\x1B[38;2;8;21;32;48;2;5;15;26m▀\x1B[38;2;10;22;33;48;2;4;15;26m▀\x1B[38;2;8;21;32m▀\x1B[38;2;6;20;32m▀▀▀\x1B[48;2;4;15;25m▀\x1B[38;2;6;20;33;48;2;3;14;24m▀\x1B[38;2;3;13;23;48;2;13;35;52m▀\x1B[38;2;13;34;52;48;2;29;71;99m▀\x1B[0;38;2;31;74;103m▀\x1B[0m \x1B[0m',
	'     \x1B[38;2;32;77;107m▀\x1B[38;2;23;56;79m▀\x1B[38;2;16;41;59;48;2;33;83;113m▀\x1B[38;2;14;36;53;48;2;31;77;106m▀\x1B[38;2;14;36;54;48;2;31;77;105m▀▀▀▀▀▀▀▀▀▀▀▀\x1B[38;2;13;36;53;48;2;31;77;106m▀\x1B[38;2;16;41;59;48;2;33;83;113m▀\x1B[0;38;2;23;55;78m▀\x1B[38;2;32;77;107m▀\x1B[0m   \x1B[0m',
]

const EMPTY_STATE_INFO: Record<string, string[]> = {
	server: [
		'',
		'',
		'\x1B[1;38;2;61;214;245mSOLARA\x1B[1;38;2;255;181;90m LAUNCHER\x1B[0m \x1B[90m· SERVER CONSOLE\x1B[0m',
		'\x1B[90m────────────────────────────────────────────\x1B[0m',
		'\x1B[38;2;61;214;245mStatus\x1B[0m  : Waiting for server startup',
		'\x1B[38;2;255;181;90mAction\x1B[0m  : Press \x1B[1;97mStart\x1B[0m to launch your server',
		'          and stream live logs here.',
		'',
		'\x1B[90mReady for live output...\x1B[0m',
	],
	instance: [
		'',
		'',
		'\x1B[1;38;2;61;214;245mSOLARA\x1B[1;38;2;255;181;90m LAUNCHER\x1B[0m \x1B[90m· LIVE CONSOLE\x1B[0m',
		'\x1B[90m────────────────────────────────────────────\x1B[0m',
		'\x1B[38;2;61;214;245mStatus\x1B[0m  : No active Minecraft session',
		'\x1B[38;2;255;181;90mAction\x1B[0m  : Click \x1B[1;38;2;61;214;245mLaunch Game\x1B[0m in the top right',
		'          to start streaming live game logs!',
		'',
		'\x1B[90mTip: Switch logs above to view past sessions.\x1B[0m',
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
		for (let i = 0; i < SOLARA_LOGO.length; i++) {
			const right = info[i] ? `   ${info[i]}` : ''
			terminal.value.writeln(`${SOLARA_LOGO[i]}${right}`)
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

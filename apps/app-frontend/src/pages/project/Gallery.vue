<template>
	<div class="gallery">
		<div
			v-for="(image, index) in filteredGallery"
			:key="image.url"
			class="gallery-item rounded-2xl bg-bg-raised transition-transform duration-150 hover:-translate-y-0.5"
		>
			<a class="block cursor-pointer overflow-hidden" @click="expandImage(image, index)">
				<img :src="image.url" :alt="image.title" class="gallery-image transition-transform duration-200 hover:scale-[1.02]" />
			</a>
			<div class="gallery-body">
				<h3 class="m-0 mb-1 text-sm font-bold text-contrast">{{ image.title }}</h3>
				<p v-if="image.description" class="m-0 text-xs text-secondary leading-relaxed">
					{{ image.description }}
				</p>
			</div>
			<span class="gallery-time flex items-center gap-1.5 text-xs text-secondary/80">
				<CalendarIcon class="size-3.5 text-brand" />
				{{ formatDate(new Date(image.created)) }}
			</span>
		</div>
	</div>
	<ImageViewerEditor
		ref="galleryViewer"
		:items="galleryViewerItems"
		editor="disabled"
		@navigate="trackGalleryNavigation"
	>
		<template #actions="{ item }">
			<Button
				type="quiet"
				class="!w-9 !rounded-full !p-0"
				aria-label="Open image in new tab"
				@click="openUrl(item.src)"
			>
				<ExternalIcon aria-hidden="true" />
			</Button>
		</template>
	</ImageViewerEditor>
</template>

<script setup>
import { CalendarIcon, ExternalIcon } from '@modrinth/assets'
import { Button, ImageViewerEditor, useFormatDateTime } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'

const MC_SERVER_BANNER_NAME = '__mc_server_banner__'

const formatDate = useFormatDateTime({
	year: 'numeric',
	month: 'long',
	day: 'numeric',
})

const props = defineProps({
	project: {
		type: Object,
		default: () => ({}),
	},
})

const filteredGallery = computed(
	() => props.project.gallery?.filter((img) => img.title !== MC_SERVER_BANNER_NAME) ?? [],
)

const galleryViewer = ref()
const galleryViewerItems = computed(() =>
	filteredGallery.value.map((image) => ({
		id: image.url,
		src: image.raw_url ?? 'https://cdn.modrinth.com/placeholder-banner.svg',
		alt: image.title || 'Gallery image',
		title: image.title,
		description: image.description,
	})),
)

const expandImage = (item, index) => {
	galleryViewer.value?.show(index)
	trackEvent('GalleryImageExpand', {
		project_id: props.project.id,
		url: item.url,
	})
}

function trackGalleryNavigation(item, _index, direction) {
	trackEvent(direction === 'next' ? 'GalleryImageNext' : 'GalleryImagePrevious', {
		project_id: props.project.id,
		url: item.id,
	})
}
</script>

<style scoped lang="scss">
.gallery {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
	width: 100%;
	gap: 1rem;
}

.gallery-item {
	padding: 0;
	overflow: hidden;
	margin: 0;
	display: flex;
	flex-direction: column;

	.gallery-image {
		width: 100%;
		aspect-ratio: 2/1;
		object-fit: cover;
		object-position: center;
	}

	.gallery-body {
		flex-grow: 1;
		padding: 1rem;
	}

	.gallery-time {
		padding: 0 1rem 1rem;
		vertical-align: center;
	}
}
</style>

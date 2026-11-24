<template>
	<div class="rounded-2xl bg-bg-raised p-5">
		<ProjectPageDescription :description="project.body" />
	</div>
</template>

<script setup>
import { ProjectPageDescription } from '@modrinth/ui'

defineProps({
	project: {
		type: Object,
		default: () => {},
	},
})
</script>

<script>
export default {
	name: 'Description',
}
</script>

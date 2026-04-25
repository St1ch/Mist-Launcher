<template>
	<Transition name="splash-fade" @after-leave="onAfterLeave">
		<div v-if="!doneLoading" class="splash-screen dark">
			<div class="app-logo-wrapper" data-tauri-drag-region>
				<div class="mist-splash-logo" data-tauri-drag-region>
					<img :src="mistLogo" alt="" class="app-logo-icon" />
					<span data-tauri-drag-region class="app-logo-text">Launcher</span>
				</div>
				<ProgressBar class="loading-bar" :progress="Math.min(loadingProgress, 100)" />
				<span v-if="message" class="loading-message">{{ message }}</span>
			</div>
			<div class="gradient-bg" data-tauri-drag-region></div>
			<div class="mist-orb mist-orb-one"></div>
			<div class="mist-orb mist-orb-two"></div>
			<div class="base-bg"></div>
		</div>
	</Transition>
</template>

<script setup>
import { injectLoadingState } from '@modrinth/ui'
import { ref, watch } from 'vue'

import mistLogo from '@/assets/mist-logo.png'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { loading_listener } from '@/helpers/events.js'

const doneLoading = ref(false)
const loadingProgress = ref(0)
const message = ref()

const MIN_DISPLAY_MS = 500
const mountedAt = Date.now()

const loading = injectLoadingState()

function onAfterLeave() {
	loading.setEnabled(true)
}

watch(
	[loading.barEnabled, loading.pending],
	([barEnabled, pending]) => {
		if (barEnabled) {
			return
		}

		if (pending) {
			loadingProgress.value = 0
			fakeLoadingIncrease()
			return
		}

		const elapsed = Date.now() - mountedAt
		const delay = Math.max(0, MIN_DISPLAY_MS - elapsed)

		setTimeout(() => {
			if (loading.pending.value) {
				return
			}
			doneLoading.value = true
		}, delay)
	},
	{ immediate: true },
)

function fakeLoadingIncrease() {
	if (loadingProgress.value < 95) {
		setTimeout(() => {
			loadingProgress.value += 1
			fakeLoadingIncrease()
		}, 5)
	}
}

loading_listener(async (e) => {
	if (e.event.type === 'directory_move') {
		loadingProgress.value = 100 * (e.fraction ?? 1)
		message.value = 'Updating launcher directory...'
	} else if (e.event.type === 'checking_for_updates') {
		loadingProgress.value = 100 * (e.fraction ?? 1)
		message.value = 'Checking for updates...'
	}
})
</script>

<style scoped lang="scss">
.splash-screen {
	position: fixed;
	inset: 0;
	z-index: 10000;
	overflow: hidden;
}

.splash-fade-leave-active {
	transition: opacity 0.3s ease-in-out;
}

.splash-fade-leave-to {
	opacity: 0;
}

.app-logo-wrapper {
	position: absolute;
	inset: 0;
	display: flex;
	flex-direction: column;
	justify-content: center;
	align-items: center;
	gap: 1rem;
	z-index: 9998;
}

.mist-splash-logo {
	display: flex;
	align-items: center;
	gap: 0.9rem;
	color: var(--color-contrast);
}

.app-logo-icon {
	width: 3.25rem;
	height: 3.25rem;
	object-fit: contain;
	border-radius: 0.85rem;
	filter: drop-shadow(0 0 1rem rgba(143, 255, 210, 0.36));
}

.app-logo-text {
	font-size: 2rem;
	font-weight: 850;
	letter-spacing: 0.02em;
	line-height: 1;
}

.loading-bar {
	max-width: 20rem;
}

.loading-message {
	color: var(--color-secondary);
	font-weight: 700;
}

.gradient-bg {
	position: absolute;
	inset: 0;
	background:
		radial-gradient(circle at 50% 42%, rgba(143, 255, 210, 0.2) 0%, rgba(143, 255, 210, 0) 34%),
		linear-gradient(180deg, rgba(18, 28, 30, 0.62) 0%, rgba(9, 13, 18, 0.84) 100%);
	z-index: 9997;
}

.mist-orb {
	position: absolute;
	border-radius: 999px;
	background: rgba(143, 255, 210, 0.2);
	filter: blur(3rem);
	z-index: 9996;
}

.mist-orb-one {
	width: 26rem;
	height: 26rem;
	top: 12%;
	left: 18%;
}

.mist-orb-two {
	width: 18rem;
	height: 18rem;
	right: 16%;
	bottom: 18%;
	opacity: 0.65;
}

.base-bg {
	position: absolute;
	inset: 0;
	background: var(--color-bg);
	z-index: 9995;
}
</style>

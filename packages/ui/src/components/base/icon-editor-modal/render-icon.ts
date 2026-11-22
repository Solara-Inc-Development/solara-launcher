import type { IconConfig } from './types'

export type StudioMode = 'icon' | 'monogram' | 'upload'
export type StudioBackdropMode = 'gradient' | 'solid'
export type StudioPattern = 'none' | 'dots' | 'diagonal' | 'grid'
export type StudioFrame = 'none' | 'border' | 'corners' | 'ring'
export type StudioFont = 'bold' | 'mono' | 'serif'
export type GradientAngle = 180 | 135 | 90 | 45

export interface StudioRenderOptions {
	mode: StudioMode
	backdropMode: StudioBackdropMode
	primaryColor: string
	secondaryColor: string
	angle: GradientAngle
	pattern: StudioPattern
	frame: StudioFrame
	scale: number // 0.55..0.95
	shadow: boolean
	symbolAsset?: string
	uploadAsset?: string | null
	monogramText?: string
	monogramSubtext?: string
	monogramFont?: StudioFont
	monogramColor?: string
}

function drawBackground(
	ctx: CanvasRenderingContext2D,
	size: number,
	opts: StudioRenderOptions,
) {
	if (opts.backdropMode === 'solid') {
		ctx.fillStyle = opts.primaryColor
	} else {
		const rad = ((opts.angle - 90) * Math.PI) / 180
		const half = size / 2
		const dx = Math.cos(rad) * half
		const dy = Math.sin(rad) * half
		const grad = ctx.createLinearGradient(half - dx, half - dy, half + dx, half + dy)
		grad.addColorStop(0, opts.primaryColor)
		grad.addColorStop(1, opts.secondaryColor)
		ctx.fillStyle = grad
	}
	ctx.fillRect(0, 0, size, size)

	// Subtle top specular sheen for depth
	const sheen = ctx.createLinearGradient(0, 0, 0, size * 0.55)
	sheen.addColorStop(0, 'rgba(255, 255, 255, 0.14)')
	sheen.addColorStop(1, 'rgba(255, 255, 255, 0)')
	ctx.fillStyle = sheen
	ctx.fillRect(0, 0, size, size * 0.55)
}

function drawPattern(
	ctx: CanvasRenderingContext2D,
	size: number,
	pattern: StudioPattern,
) {
	if (pattern === 'none') return
	ctx.save()
	if (pattern === 'dots') {
		ctx.fillStyle = 'rgba(255, 255, 255, 0.11)'
		const step = 32
		for (let y = step / 2; y < size; y += step) {
			for (let x = step / 2; x < size; x += step) {
				ctx.beginPath()
				ctx.arc(x, y, 2.4, 0, Math.PI * 2)
				ctx.fill()
			}
		}
	} else if (pattern === 'diagonal') {
		ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)'
		ctx.lineWidth = 3
		const step = 36
		for (let offset = -size; offset < size * 2; offset += step) {
			ctx.beginPath()
			ctx.moveTo(offset, 0)
			ctx.lineTo(offset + size, size)
			ctx.stroke()
		}
	} else if (pattern === 'grid') {
		ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)'
		ctx.lineWidth = 1.5
		const step = 48
		for (let p = step; p < size; p += step) {
			ctx.beginPath()
			ctx.moveTo(p, 0)
			ctx.lineTo(p, size)
			ctx.stroke()
			ctx.beginPath()
			ctx.moveTo(0, p)
			ctx.lineTo(size, p)
			ctx.stroke()
		}
	}
	ctx.restore()
}

function drawFrame(
	ctx: CanvasRenderingContext2D,
	size: number,
	frame: StudioFrame,
) {
	if (frame === 'none') return
	ctx.save()
	if (frame === 'border') {
		const inset = 22
		const radius = 64
		ctx.strokeStyle = 'rgba(255, 255, 255, 0.34)'
		ctx.lineWidth = 6
		ctx.beginPath()
		ctx.roundRect(inset, inset, size - inset * 2, size - inset * 2, radius)
		ctx.stroke()
	} else if (frame === 'corners') {
		const inset = 34
		const len = 64
		ctx.strokeStyle = 'rgba(255, 255, 255, 0.55)'
		ctx.lineWidth = 10
		ctx.lineCap = 'round'
		ctx.lineJoin = 'round'

		const corners: [number, number, number, number][] = [
			[inset, inset, 1, 1],
			[size - inset, inset, -1, 1],
			[inset, size - inset, 1, -1],
			[size - inset, size - inset, -1, -1],
		]
		for (const [cx, cy, sx, sy] of corners) {
			ctx.beginPath()
			ctx.moveTo(cx + sx * len, cy)
			ctx.lineTo(cx, cy)
			ctx.lineTo(cx, cy + sy * len)
			ctx.stroke()
		}
	} else if (frame === 'ring') {
		ctx.strokeStyle = 'rgba(255, 255, 255, 0.24)'
		ctx.lineWidth = 6
		ctx.beginPath()
		ctx.arc(size / 2, size / 2, size * 0.42, 0, Math.PI * 2)
		ctx.stroke()

		ctx.strokeStyle = 'rgba(255, 255, 255, 0.14)'
		ctx.lineWidth = 3
		ctx.beginPath()
		ctx.arc(size / 2, size / 2, size * 0.37, 0, Math.PI * 2)
		ctx.stroke()
	}
	ctx.restore()
}

async function loadImage(src: string): Promise<HTMLImageElement> {
	const img = new Image()
	img.crossOrigin = 'anonymous'
	img.src = src
	await img.decode()
	return img
}

export async function renderStudioCanvas(
	canvas: HTMLCanvasElement,
	opts: StudioRenderOptions,
): Promise<void> {
	const size = canvas.width
	const ctx = canvas.getContext('2d')
	if (!ctx) throw new Error('Could not initialize icon studio canvas.')

	ctx.clearRect(0, 0, size, size)
	drawBackground(ctx, size, opts)
	drawPattern(ctx, size, opts.pattern)
	drawFrame(ctx, size, opts.frame)

	ctx.save()
	if (opts.shadow) {
		ctx.shadowColor = 'rgba(0, 0, 0, 0.42)'
		ctx.shadowBlur = size * 0.065
		ctx.shadowOffsetY = size * 0.025
	}

	if (opts.mode === 'monogram') {
		const rawText = (opts.monogramText ?? 'SOL').trim().toUpperCase().slice(0, 5) || 'SOL'
		const sub = (opts.monogramSubtext ?? '').trim().toUpperCase().slice(0, 12)
		const fontFamily =
			opts.monogramFont === 'mono'
				? '"JetBrains Mono", "Cascadia Code", ui-monospace, monospace'
				: opts.monogramFont === 'serif'
					? 'Georgia, "Times New Roman", serif'
					: 'Inter, "Segoe UI", system-ui, sans-serif'

		const baseFontSize =
			rawText.length <= 2
				? size * 0.42
				: rawText.length === 3
					? size * 0.33
					: rawText.length === 4
						? size * 0.26
						: size * 0.22
		const fontSize = Math.round(baseFontSize * Math.max(0.65, Math.min(1.15, opts.scale / 0.78)))

		ctx.fillStyle = opts.monogramColor || '#ffffff'
		ctx.textAlign = 'center'
		ctx.textBaseline = 'middle'
		ctx.font = `800 ${fontSize}px ${fontFamily}`

		const centerY = sub ? size * 0.46 : size * 0.52
		ctx.fillText(rawText, size / 2, centerY, size * 0.84)

		if (sub) {
			ctx.shadowBlur = 0
			ctx.shadowOffsetY = 0
			const subSize = Math.round(size * 0.085)
			ctx.font = `700 ${subSize}px Inter, system-ui, sans-serif`
			ctx.fillStyle = 'rgba(255, 255, 255, 0.82)'
			ctx.fillText(sub, size / 2, size * 0.75, size * 0.78)
		}
	} else {
		const src = opts.mode === 'upload' ? opts.uploadAsset : opts.symbolAsset
		if (src) {
			try {
				const img = await loadImage(src)
				if (opts.mode === 'upload') {
					// Cover-crop uploaded image smoothly inside scale box
					const target = size * opts.scale
					const offset = (size - target) / 2
					const imgAspect = img.naturalWidth / Math.max(1, img.naturalHeight)
					let sx = 0
					let sy = 0
					let sw = img.naturalWidth
					let sh = img.naturalHeight
					if (imgAspect > 1) {
						sw = img.naturalHeight
						sx = (img.naturalWidth - sw) / 2
					} else {
						sh = img.naturalWidth
						sy = (img.naturalHeight - sh) / 2
					}
					const clipRadius = opts.scale < 0.98 ? target * 0.16 : 0
					if (clipRadius > 0) {
						ctx.beginPath()
						ctx.roundRect(offset, offset, target, target, clipRadius)
						ctx.clip()
					}
					ctx.drawImage(img, sx, sy, sw, sh, offset, offset, target, target)
				} else {
					const target = size * opts.scale
					const offset = (size - target) / 2
					ctx.drawImage(img, offset, offset, target, target)
				}
			} catch {
				// Ignore transient decode failures
			}
		}
	}
	ctx.restore()
}

export async function renderStudioIconDataUrl(opts: StudioRenderOptions, size = 512): Promise<string> {
	const canvas = document.createElement('canvas')
	canvas.width = size
	canvas.height = size
	await renderStudioCanvas(canvas, opts)
	return canvas.toDataURL('image/png')
}

export async function renderIcon(config: IconConfig, symbolAsset: string): Promise<File> {
	const primaryColor =
		config.background.type === 'color' ? config.background.value : config.background.top_color
	const secondaryColor =
		config.background.type === 'color' ? config.background.value : config.background.bottom_color
	const canvas = document.createElement('canvas')
	canvas.width = 512
	canvas.height = 512
	await renderStudioCanvas(canvas, {
		mode: 'icon',
		backdropMode: config.background.type === 'color' ? 'solid' : 'gradient',
		primaryColor,
		secondaryColor,
		angle: 180,
		pattern: 'none',
		frame: 'none',
		scale: 0.74,
		shadow: true,
		symbolAsset,
	})

	const blob = await new Promise<Blob>((resolve, reject) => {
		canvas.toBlob((result) => {
			if (result) resolve(result)
			else reject(new Error('Could not encode the icon image.'))
		}, 'image/png')
	})
	return new File([blob], 'instance-icon.png', { type: 'image/png' })
}

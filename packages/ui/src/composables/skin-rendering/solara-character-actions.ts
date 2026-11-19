import * as THREE from 'three'

export const SOLARA_ACTION_NAMES = [
	'solara_wave',
	'solara_sit_stand',
	'solara_cheer_jump',
	'solara_stretch',
	'solara_look_around',
	'solara_salute',
] as const

export type SolaraActionName = (typeof SOLARA_ACTION_NAMES)[number]

export function isSolaraActionName(name: string): name is SolaraActionName {
	return (SOLARA_ACTION_NAMES as readonly string[]).includes(name)
}

interface ActionDefinition {
	duration: number
	fadeIn: number
	fadeOut: number
}

export const SOLARA_ACTION_DEFS: Record<SolaraActionName, ActionDefinition> = {
	solara_wave: { duration: 3.8, fadeIn: 0.32, fadeOut: 0.38 },
	solara_sit_stand: { duration: 6.2, fadeIn: 0.55, fadeOut: 0.55 },
	solara_cheer_jump: { duration: 3.2, fadeIn: 0.25, fadeOut: 0.35 },
	solara_stretch: { duration: 4.4, fadeIn: 0.45, fadeOut: 0.45 },
	solara_look_around: { duration: 4.6, fadeIn: 0.38, fadeOut: 0.4 },
	solara_salute: { duration: 3.4, fadeIn: 0.3, fadeOut: 0.38 },
}

interface RigBones {
	orbit: THREE.Object3D | null
	body: THREE.Object3D | null
	head: THREE.Object3D | null
	rightArm: THREE.Object3D | null
	leftArm: THREE.Object3D | null
	rightLeg: THREE.Object3D | null
	leftLeg: THREE.Object3D | null
	orbitBaseY: number
}

function clamp01(v: number): number {
	return Math.max(0, Math.min(1, v))
}

function smoothstep(edge0: number, edge1: number, x: number): number {
	const t = clamp01((x - edge0) / Math.max(1e-5, edge1 - edge0))
	return t * t * (3 - 2 * t)
}

function blendAngle(current: number, target: number, weight: number): number {
	return current * (1 - weight) + target * weight
}

function blendBoneRotation(
	bone: THREE.Object3D | null,
	rx: number,
	ry: number,
	rz: number,
	weight: number,
) {
	if (!bone) return
	bone.rotation.x = blendAngle(bone.rotation.x, rx, weight)
	bone.rotation.y = blendAngle(bone.rotation.y, ry, weight)
	bone.rotation.z = blendAngle(bone.rotation.z, rz, weight)
}

export class SolaraCharacterActionController {
	private bones: RigBones = {
		orbit: null,
		body: null,
		head: null,
		rightArm: null,
		leftArm: null,
		rightLeg: null,
		leftLeg: null,
		orbitBaseY: 0.75,
	}

	private activeAction: SolaraActionName | null = null
	private elapsed = 0
	private onComplete: (() => void) | null = null

	attach(scene: THREE.Object3D) {
		this.cleanup()

		const orbit = scene.getObjectByName('Orbit') ?? null
		const body = scene.getObjectByName('Body') ?? null
		const head = scene.getObjectByName('Head') ?? null
		const rightArm = scene.getObjectByName('Right_Arm') ?? null
		const leftArm = scene.getObjectByName('Left_Arm') ?? null
		const rightLeg = scene.getObjectByName('Right_Leg') ?? null
		const leftLeg = scene.getObjectByName('Left_Leg') ?? null

		this.bones = {
			orbit,
			body,
			head,
			rightArm,
			leftArm,
			rightLeg,
			leftLeg,
			orbitBaseY: orbit ? orbit.position.y : 0.75,
		}
	}

	get currentAction(): SolaraActionName | null {
		return this.activeAction
	}

	get isPlaying(): boolean {
		return this.activeAction !== null
	}

	play(action: SolaraActionName, onComplete?: () => void) {
		this.stop()
		this.activeAction = action
		this.elapsed = 0
		this.onComplete = onComplete ?? null
	}

	stop() {
		if (this.bones.orbit) {
			this.bones.orbit.position.y = this.bones.orbitBaseY
		}
		this.activeAction = null
		this.elapsed = 0
		this.onComplete = null
	}

	/**
	 * Must be called each frame AFTER `mixer.update(delta)` so procedural
	 * pose blending layers cleanly over the underlying idle pose.
	 */
	update(delta: number) {
		const action = this.activeAction
		if (!action) return

		const def = SOLARA_ACTION_DEFS[action]
		this.elapsed += delta

		if (this.elapsed >= def.duration) {
			const done = this.onComplete
			this.stop()
			done?.()
			return
		}

		const t = this.elapsed
		const fadeIn = smoothstep(0, def.fadeIn, t)
		const fadeOut = 1 - smoothstep(def.duration - def.fadeOut, def.duration, t)
		const w = clamp01(Math.min(fadeIn, fadeOut))

		const { orbit, body, head, rightArm, leftArm, rightLeg, leftLeg, orbitBaseY } = this.bones

		// Ensure Orbit Y stays grounded on standing poses (e.g., if interrupting Sit or Jump)
		if (orbit && action !== 'solara_sit_stand' && action !== 'solara_cheer_jump') {
			orbit.position.y = orbitBaseY
		}

		switch (action) {
			case 'solara_wave': {
				const waveOsc = Math.sin(t * 10.5) * 0.34
				blendBoneRotation(rightArm, 2.58, 0.15, 0.42 + waveOsc, w)
				blendBoneRotation(leftArm, 0.1, 0, -0.14, w)
				blendBoneRotation(head, 0.06, -0.12, -0.14 + waveOsc * 0.12, w)
				blendBoneRotation(body, 0, -0.08, -0.04 * Math.sin(t * 5.25), w)
				blendBoneRotation(rightLeg, 0, 0, 0, w)
				blendBoneRotation(leftLeg, 0, 0, 0, w)
				break
			}

			case 'solara_sit_stand': {
				const legSwing = Math.sin(t * 3.1) * 0.1
				const lookAround = Math.sin((t - 0.8) * 1.35) * 0.32

				if (orbit) {
					const sitDrop = 0.44 * w
					orbit.position.y = orbitBaseY - sitDrop
				}
				blendBoneRotation(rightLeg, 1.32 + legSwing, 0, 0.14, w)
				blendBoneRotation(leftLeg, 1.32 - legSwing, 0, -0.14, w)
				blendBoneRotation(rightArm, 0.56, 0, 0.14, w)
				blendBoneRotation(leftArm, 0.56, 0, -0.14, w)
				blendBoneRotation(body, 0.08, lookAround * 0.18, 0, w)
				blendBoneRotation(head, 0.05, lookAround, 0.04 * Math.sin(t * 1.5), w)
				break
			}

			case 'solara_cheer_jump': {
				let jumpY = 0
				let armLift = 0
				let legKick = 0
				let crouchLean = 0

				if (t < 0.48) {
					const u = Math.sin((t / 0.48) * Math.PI)
					jumpY = -0.11 * u
					crouchLean = -0.22 * u
					armLift = -0.35 * u
				} else if (t < 1.95) {
					const u = (t - 0.48) / (1.95 - 0.48)
					const arc = Math.sin(u * Math.PI)
					jumpY = 0.42 * arc
					armLift = 2.65 * Math.pow(arc, 0.65)
					legKick = 0.35 * arc
				} else {
					const u = clamp01((t - 1.95) / 0.55)
					const bounce = Math.sin(u * Math.PI)
					jumpY = -0.07 * bounce
					crouchLean = -0.12 * bounce
				}

				if (orbit) {
					orbit.position.y = orbitBaseY + jumpY
				}
				blendBoneRotation(rightArm, Math.max(0, armLift), 0, 0.32 * clamp01(armLift / 2.2), w)
				blendBoneRotation(leftArm, Math.max(0, armLift), 0, -0.32 * clamp01(armLift / 2.2), w)
				blendBoneRotation(rightLeg, legKick, 0, 0.12 * legKick, w)
				blendBoneRotation(leftLeg, -legKick * 0.85, 0, -0.12 * legKick, w)
				blendBoneRotation(body, crouchLean, 0, 0, w)
				blendBoneRotation(head, 0.12 * clamp01(jumpY * 2.5), 0, 0, w)
				break
			}

			case 'solara_stretch': {
				// Relaxed arms-up stretch with gentle sway & proud exhale
				const stretchPeak = Math.sin(clamp01(t / def.duration) * Math.PI)
				const sway = Math.sin(t * 2.6) * 0.08
				blendBoneRotation(rightArm, 2.75 * stretchPeak, 0, 0.28 + sway, w)
				blendBoneRotation(leftArm, 2.75 * stretchPeak, 0, -0.28 + sway, w)
				blendBoneRotation(body, 0.12 * stretchPeak, sway * 0.4, sway * 0.3, w)
				blendBoneRotation(head, 0.18 * stretchPeak, 0, sway * 0.4, w)
				blendBoneRotation(rightLeg, -0.05 * stretchPeak, 0, 0.05, w)
				blendBoneRotation(leftLeg, 0.05 * stretchPeak, 0, -0.05, w)
				break
			}

			case 'solara_look_around': {
				// Curious explorer look left -> look right with relaxed balanced stance
				const lookSweep = Math.sin((t - 0.35) * 1.65) * 0.52
				const tilt = Math.sin(t * 1.65) * 0.07
				blendBoneRotation(head, 0.05 * Math.sin(t * 2.2), lookSweep, tilt, w)
				blendBoneRotation(body, -0.03, lookSweep * 0.28, 0, w)
				blendBoneRotation(rightArm, 0.18 + lookSweep * 0.12, 0, 0.14, w)
				blendBoneRotation(leftArm, 0.18 - lookSweep * 0.12, 0, -0.14, w)
				blendBoneRotation(rightLeg, -0.06, 0, 0.06, w)
				blendBoneRotation(leftLeg, 0.06, 0, -0.06, w)
				break
			}

			case 'solara_salute': {
				// Crisp friendly salute with subtle confident nod
				const nod = t > 1.0 && t < 2.2 ? Math.sin(((t - 1.0) / 1.2) * Math.PI) * -0.14 : 0
				blendBoneRotation(rightArm, 2.15, 0.35, -0.45, w)
				blendBoneRotation(leftArm, -0.08, 0, -0.12, w)
				blendBoneRotation(head, nod, -0.08, 0.04, w)
				blendBoneRotation(body, 0.03, -0.05, 0, w)
				blendBoneRotation(rightLeg, 0, 0, 0.04, w)
				blendBoneRotation(leftLeg, 0, 0, -0.04, w)
				break
			}
		}
	}

	cleanup() {
		this.stop()
		this.bones = {
			orbit: null,
			body: null,
			head: null,
			rightArm: null,
			leftArm: null,
			rightLeg: null,
			leftLeg: null,
			orbitBaseY: 0.75,
		}
	}
}

import * as THREE from 'three'

export const SOLARA_ACTION_NAMES = [
	'solara_eat_golden_apple',
	'solara_hold_sword',
	'solara_hold_pickaxe',
	'solara_hold_torch',
	'solara_wave',
	'solara_sit_stand',
	'solara_cheer_jump',
] as const

export type SolaraActionName = (typeof SOLARA_ACTION_NAMES)[number]

export function isSolaraActionName(name: string): name is SolaraActionName {
	return (SOLARA_ACTION_NAMES as readonly string[]).includes(name)
}

type PropKind = 'golden_apple' | 'diamond_sword' | 'diamond_pickaxe' | 'torch'

interface ActionDefinition {
	duration: number
	fadeIn: number
	fadeOut: number
	prop?: PropKind
}

export const SOLARA_ACTION_DEFS: Record<SolaraActionName, ActionDefinition> = {
	solara_eat_golden_apple: { duration: 4.8, fadeIn: 0.38, fadeOut: 0.42, prop: 'golden_apple' },
	solara_hold_sword: { duration: 5.0, fadeIn: 0.35, fadeOut: 0.42, prop: 'diamond_sword' },
	solara_hold_pickaxe: { duration: 5.0, fadeIn: 0.35, fadeOut: 0.42, prop: 'diamond_pickaxe' },
	solara_hold_torch: { duration: 4.8, fadeIn: 0.35, fadeOut: 0.42, prop: 'torch' },
	solara_wave: { duration: 3.8, fadeIn: 0.32, fadeOut: 0.38 },
	solara_sit_stand: { duration: 6.2, fadeIn: 0.55, fadeOut: 0.55 },
	solara_cheer_jump: { duration: 3.2, fadeIn: 0.25, fadeOut: 0.35 },
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

interface PropParticle {
	mesh: THREE.Mesh
	vx: number
	vy: number
	vz: number
	age: number
	life: number
}

interface PropEntry {
	group: THREE.Group
	particles: PropParticle[]
	particleAccumulator: number
}

function clamp01(v: number): number {
	return Math.max(0, Math.min(1, v))
}

function smoothstep(edge0: number, edge1: number, x: number): number {
	const t = clamp01((x - edge0) / Math.max(1e-5, edge1 - edge0))
	return t * t * (3 - 2 * t)
}

function createVoxelMaterial(color: number, emissive = 0x000000, emissiveIntensity = 0) {
	return new THREE.MeshStandardMaterial({
		color,
		emissive,
		emissiveIntensity,
		roughness: 0.45,
		metalness: emissiveIntensity > 0 ? 0.15 : 0.08,
		flatShading: true,
		toneMapped: false,
	})
}

function addVoxelBox(
	parent: THREE.Group,
	w: number,
	h: number,
	d: number,
	x: number,
	y: number,
	z: number,
	mat: THREE.Material,
): THREE.Mesh {
	const geom = new THREE.BoxGeometry(w, h, d)
	const mesh = new THREE.Mesh(geom, mat)
	mesh.position.set(x, y, z)
	mesh.userData.isSolaraProp = true
	parent.add(mesh)
	return mesh
}

function buildGoldenAppleProp(): PropEntry {
	const group = new THREE.Group()
	group.name = 'SolaraProp_GoldenApple'
	group.userData.isSolaraProp = true
	// Held neatly right in front of the fist
	group.position.set(-0.02, -0.58, -0.11)

	const goldMain = createVoxelMaterial(0xf7c927, 0xffae00, 0.32)
	const goldHighlight = createVoxelMaterial(0xfff199, 0xffe066, 0.45)
	const goldShade = createVoxelMaterial(0xd6920a, 0x995800, 0.18)
	const stemMat = createVoxelMaterial(0x6b4423)
	const leafMat = createVoxelMaterial(0x52c442, 0x228822, 0.12)

	// Main chunky voxel apple body
	addVoxelBox(group, 0.2, 0.18, 0.2, 0, 0, 0, goldMain)
	// Rounded top/bottom voxel lobes
	addVoxelBox(group, 0.16, 0.21, 0.16, 0, 0.005, 0, goldMain)
	// Bright golden glint pixel
	addVoxelBox(group, 0.06, 0.07, 0.04, -0.05, 0.035, -0.095, goldHighlight)
	// Deeper golden bottom shadow
	addVoxelBox(group, 0.14, 0.04, 0.15, 0, -0.095, 0, goldShade)
	// Stem
	addVoxelBox(group, 0.035, 0.08, 0.035, 0, 0.135, 0, stemMat)
	// Tiny crisp green leaf
	addVoxelBox(group, 0.07, 0.035, 0.04, 0.045, 0.14, -0.01, leafMat)

	// Golden crumb particles that pop on each bite
	const particles: PropParticle[] = []
	const crumbMat = createVoxelMaterial(0xffe066, 0xffaa00, 0.5)
	const crumbGeom = new THREE.BoxGeometry(0.032, 0.032, 0.032)
	for (let i = 0; i < 6; i++) {
		const mesh = new THREE.Mesh(crumbGeom, crumbMat)
		mesh.visible = false
		mesh.userData.isSolaraProp = true
		group.add(mesh)
		particles.push({ mesh, vx: 0, vy: 0, vz: 0, age: 1, life: 0.45 })
	}

	group.visible = false
	return { group, particles, particleAccumulator: 0 }
}

function buildDiamondSwordProp(): PropEntry {
	const group = new THREE.Group()
	group.name = 'SolaraProp_DiamondSword'
	group.userData.isSolaraProp = true
	group.position.set(0, -0.57, -0.08)
	// Point blade forward-upward like Minecraft third-person hold
	group.rotation.set(-Math.PI / 2.8, 0, 0)

	const handleMat = createVoxelMaterial(0x6e4726)
	const guardMat = createVoxelMaterial(0x14707a, 0x0b4c57, 0.15)
	const diamondMain = createVoxelMaterial(0x3be3d8, 0x1ac8db, 0.28)
	const diamondEdge = createVoxelMaterial(0x8ffaf4, 0x4ef0e8, 0.42)
	const pommelMat = createVoxelMaterial(0x28b5b5, 0x16808a, 0.2)

	// Pommel & grip
	addVoxelBox(group, 0.07, 0.07, 0.055, 0, -0.14, 0, pommelMat)
	addVoxelBox(group, 0.05, 0.18, 0.045, 0, -0.02, 0, handleMat)
	// Crossguard
	addVoxelBox(group, 0.24, 0.06, 0.06, 0, 0.09, 0, guardMat)
	addVoxelBox(group, 0.07, 0.08, 0.065, 0, 0.09, 0, diamondMain)
	// Tapered voxel blade
	addVoxelBox(group, 0.1, 0.48, 0.04, 0, 0.36, 0, diamondMain)
	addVoxelBox(group, 0.045, 0.44, 0.046, 0, 0.36, 0, diamondEdge)
	addVoxelBox(group, 0.065, 0.09, 0.038, 0, 0.63, 0, diamondEdge)
	addVoxelBox(group, 0.035, 0.05, 0.034, 0, 0.69, 0, diamondEdge)

	group.visible = false
	return { group, particles: [], particleAccumulator: 0 }
}

function buildDiamondPickaxeProp(): PropEntry {
	const group = new THREE.Group()
	group.name = 'SolaraProp_DiamondPickaxe'
	group.userData.isSolaraProp = true
	group.position.set(0, -0.57, -0.08)
	group.rotation.set(-Math.PI / 2.9, 0, 0)

	const handleMat = createVoxelMaterial(0x734b28)
	const handleLight = createVoxelMaterial(0x916236)
	const diamondMain = createVoxelMaterial(0x3be3d8, 0x1ac8db, 0.28)
	const diamondTip = createVoxelMaterial(0x8ffaf4, 0x4ef0e8, 0.42)

	// Wooden shaft
	addVoxelBox(group, 0.055, 0.64, 0.055, 0, 0.22, 0, handleMat)
	addVoxelBox(group, 0.06, 0.08, 0.06, 0, -0.1, 0, handleLight)
	// Curved diamond pick head along Z (pointing forward-back so swings strike forward!)
	addVoxelBox(group, 0.065, 0.09, 0.28, 0, 0.54, 0, diamondMain)
	// Forward curved prongs
	addVoxelBox(group, 0.06, 0.08, 0.11, 0, 0.49, -0.17, diamondMain)
	addVoxelBox(group, 0.055, 0.09, 0.08, 0, 0.42, -0.24, diamondTip)
	// Back curved prongs
	addVoxelBox(group, 0.06, 0.08, 0.11, 0, 0.49, 0.17, diamondMain)
	addVoxelBox(group, 0.055, 0.09, 0.08, 0, 0.42, 0.24, diamondTip)

	group.visible = false
	return { group, particles: [], particleAccumulator: 0 }
}

function buildTorchProp(): PropEntry {
	const group = new THREE.Group()
	group.name = 'SolaraProp_Torch'
	group.userData.isSolaraProp = true
	group.position.set(0, -0.56, -0.1)
	group.rotation.set(-Math.PI / 5.5, 0, 0)

	const woodMat = createVoxelMaterial(0x7d5530)
	const coalMat = createVoxelMaterial(0x3d2c1e)
	const flameOuter = createVoxelMaterial(0xff8c1a, 0xff6a00, 0.75)
	const flameCore = createVoxelMaterial(0xfff078, 0xffd23f, 0.95)

	// Torch handle
	addVoxelBox(group, 0.075, 0.42, 0.075, 0, 0.13, 0, woodMat)
	addVoxelBox(group, 0.08, 0.06, 0.08, 0, 0.35, 0, coalMat)
	// Glowing flame top
	addVoxelBox(group, 0.09, 0.1, 0.09, 0, 0.42, 0, flameOuter)
	addVoxelBox(group, 0.055, 0.07, 0.055, 0, 0.47, 0, flameCore)

	// Rising ember particles
	const particles: PropParticle[] = []
	const emberMat = createVoxelMaterial(0xffd23f, 0xff7b00, 0.9)
	const emberGeom = new THREE.BoxGeometry(0.028, 0.028, 0.028)
	for (let i = 0; i < 5; i++) {
		const mesh = new THREE.Mesh(emberGeom, emberMat)
		mesh.visible = false
		mesh.userData.isSolaraProp = true
		group.add(mesh)
		particles.push({ mesh, vx: 0, vy: 0, vz: 0, age: 1, life: 0.7 })
	}

	group.visible = false
	return { group, particles, particleAccumulator: 0 }
}

function disposePropEntry(entry: PropEntry) {
	const geometries = new Set<THREE.BufferGeometry>()
	const materials = new Set<THREE.Material>()
	entry.group.traverse((obj) => {
		const mesh = obj as THREE.Mesh
		if (!mesh.isMesh) return
		if (mesh.geometry) geometries.add(mesh.geometry)
		const mats = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
		mats.forEach((m) => m && materials.add(m))
	})
	entry.group.removeFromParent()
	geometries.forEach((g) => g.dispose())
	materials.forEach((m) => m.dispose())
}

const tempEuler = new THREE.Euler(0, 0, 0, 'YXZ')
const tempQuat = new THREE.Quaternion()

function blendBoneRotation(
	bone: THREE.Object3D | null,
	rx: number,
	ry: number,
	rz: number,
	weight: number,
) {
	if (!bone || weight <= 0.0001) return
	tempEuler.set(rx, ry, rz, 'YXZ')
	tempQuat.setFromEuler(tempEuler)
	bone.quaternion.slerp(tempQuat, clamp01(weight))
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

	private props: Partial<Record<PropKind, PropEntry>> = {}
	private activeAction: SolaraActionName | null = null
	private elapsed = 0
	private onComplete: (() => void) | null = null

	attach(root: THREE.Object3D) {
		this.cleanup()

		const orbit = root.getObjectByName('Orbit') ?? null
		const body = root.getObjectByName('Body') ?? null
		const head = root.getObjectByName('Head') ?? null
		const rightArm = root.getObjectByName('Right_Arm') ?? null
		const leftArm = root.getObjectByName('Left_Arm') ?? null
		const rightLeg = root.getObjectByName('Right_Leg') ?? null
		const leftLeg = root.getObjectByName('Left_Leg') ?? null

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

		if (rightArm) {
			const apple = buildGoldenAppleProp()
			const sword = buildDiamondSwordProp()
			const pickaxe = buildDiamondPickaxeProp()
			const torch = buildTorchProp()
			rightArm.add(apple.group, sword.group, pickaxe.group, torch.group)
			this.props = {
				golden_apple: apple,
				diamond_sword: sword,
				diamond_pickaxe: pickaxe,
				torch,
			}
		}
	}

	isActive(): boolean {
		return this.activeAction !== null
	}

	getActiveAction(): SolaraActionName | null {
		return this.activeAction
	}

	start(action: SolaraActionName, onComplete?: () => void): boolean {
		if (!this.bones.rightArm && !this.bones.orbit) {
			return false
		}
		this.hideAllProps()
		this.activeAction = action
		this.elapsed = 0
		this.onComplete = onComplete ?? null
		return true
	}

	stop() {
		this.activeAction = null
		this.elapsed = 0
		this.onComplete = null
		if (this.bones.orbit) {
			this.bones.orbit.position.y = this.bones.orbitBaseY
		}
		this.hideAllProps()
	}

	private hideAllProps() {
		for (const entry of Object.values(this.props)) {
			if (!entry) continue
			entry.group.visible = false
			entry.group.scale.setScalar(0.001)
			entry.particleAccumulator = 0
			for (const p of entry.particles) {
				p.mesh.visible = false
				p.age = p.life
			}
		}
	}

	private updateAppleParticles(entry: PropEntry, delta: number, isBiting: boolean) {
		if (isBiting) {
			entry.particleAccumulator += delta
			if (entry.particleAccumulator >= 0.085) {
				entry.particleAccumulator = 0
				const free = entry.particles.find((p) => p.age >= p.life)
				if (free) {
					free.age = 0
					free.life = 0.42 + Math.random() * 0.22
					free.mesh.position.set(
						(Math.random() - 0.5) * 0.1,
						0.02 + Math.random() * 0.05,
						-0.08,
					)
					free.vx = (Math.random() - 0.5) * 0.55
					free.vy = 0.35 + Math.random() * 0.45
					free.vz = -0.35 - Math.random() * 0.35
					free.mesh.visible = true
				}
			}
		}

		for (const p of entry.particles) {
			if (p.age >= p.life) {
				p.mesh.visible = false
				continue
			}
			p.age += delta
			p.vy -= 2.4 * delta
			p.mesh.position.x += p.vx * delta
			p.mesh.position.y += p.vy * delta
			p.mesh.position.z += p.vz * delta
			p.mesh.rotation.x += 6 * delta
			p.mesh.rotation.y += 5 * delta
			const s = Math.max(0.01, 1 - p.age / p.life)
			p.mesh.scale.setScalar(s)
		}
	}

	private updateTorchEmbers(entry: PropEntry, delta: number, active: boolean) {
		if (active) {
			entry.particleAccumulator += delta
			if (entry.particleAccumulator >= 0.16) {
				entry.particleAccumulator = 0
				const free = entry.particles.find((p) => p.age >= p.life)
				if (free) {
					free.age = 0
					free.life = 0.65 + Math.random() * 0.3
					free.mesh.position.set(
						(Math.random() - 0.5) * 0.05,
						0.49,
						(Math.random() - 0.5) * 0.05,
					)
					free.vx = (Math.random() - 0.5) * 0.12
					free.vy = 0.38 + Math.random() * 0.25
					free.vz = (Math.random() - 0.5) * 0.12
					free.mesh.visible = true
				}
			}
		}

		for (const p of entry.particles) {
			if (p.age >= p.life) {
				p.mesh.visible = false
				continue
			}
			p.age += delta
			p.mesh.position.x += p.vx * delta
			p.mesh.position.y += p.vy * delta
			p.mesh.position.z += p.vz * delta
			const s = Math.max(0.01, 1 - p.age / p.life)
			p.mesh.scale.setScalar(s)
		}
	}

	update(delta: number) {
		if (!this.activeAction) return

		const action = this.activeAction
		const def = SOLARA_ACTION_DEFS[action]
		this.elapsed += Math.min(delta, 0.05)

		if (this.elapsed >= def.duration) {
			const done = this.onComplete
			this.stop()
			done?.()
			return
		}

		const t = this.elapsed
		const wIn = smoothstep(0, def.fadeIn, t)
		const wOut = 1 - smoothstep(def.duration - def.fadeOut, def.duration, t)
		const w = wIn * wOut

		// Scale prop smoothly with envelope
		if (def.prop) {
			const propEntry = this.props[def.prop]
			if (propEntry) {
				propEntry.group.visible = w > 0.02
				const propScale = Math.max(0.001, smoothstep(0, 0.25, w))
				propEntry.group.scale.setScalar(propScale)
			}
		}

		const { orbit, body, head, rightArm, leftArm, rightLeg, leftLeg, orbitBaseY } = this.bones

		// Ensure Orbit Y stays grounded on standing actions (e.g., if interrupting Sit or Jump)
		if (orbit && action !== 'solara_sit_stand' && action !== 'solara_cheer_jump') {
			orbit.position.y = orbitBaseY
		}

		switch (action) {
			case 'solara_eat_golden_apple': {
				const eatingPhase = clamp01((t - 0.45) / (def.duration - 1.0))
				const isBiting = t > 0.5 && t < def.duration - 0.65
				const chomp = isBiting ? Math.sin(t * 16) : 0
				const biteLift = isBiting ? Math.abs(chomp) * 0.16 : 0

				// Right arm brings golden apple right up to mouth and munches!
				blendBoneRotation(
					rightArm,
					1.88 + biteLift,
					0.32 + chomp * 0.08,
					-0.54 + chomp * 0.06,
					w,
				)
				// Left arm balanced slightly forward
				blendBoneRotation(leftArm, 0.22, 0, -0.1, w)
				// Head tilts gently down toward apple & nods slightly on each chomp
				blendBoneRotation(head, -0.18 - biteLift * 0.5, -0.18, 0.06 * Math.sin(t * 4), w)
				blendBoneRotation(body, -0.04, -0.06 * Math.sin(eatingPhase * Math.PI), 0, w)
				blendBoneRotation(rightLeg, 0, 0, 0, w)
				blendBoneRotation(leftLeg, 0, 0, 0, w)

				const appleEntry = this.props.golden_apple
				if (appleEntry) {
					appleEntry.group.rotation.y = t * 1.4
					this.updateAppleParticles(appleEntry, delta, isBiting)
				}
				break
			}

			case 'solara_hold_sword': {
				// Heroic ready stance -> admire blade -> crisp slash flourish -> hold!
				const slashStart = 1.6
				const slashEnd = 2.65
				let slashOffsetX = 0
				let slashOffsetZ = 0
				let torsoTwist = 0
				if (t > slashStart && t < slashEnd) {
					const p = (t - slashStart) / (slashEnd - slashStart)
					// Wind up then swift diagonal slash
					if (p < 0.35) {
						const u = smoothstep(0, 0.35, p)
						slashOffsetX = 0.85 * u
						slashOffsetZ = 0.32 * u
						torsoTwist = 0.22 * u
					} else if (p < 0.68) {
						const u = smoothstep(0.35, 0.68, p)
						slashOffsetX = 0.85 - 1.55 * u
						slashOffsetZ = 0.32 - 0.65 * u
						torsoTwist = 0.22 - 0.48 * u
					} else {
						const u = smoothstep(0.68, 1, p)
						slashOffsetX = -0.7 * (1 - u)
						slashOffsetZ = -0.33 * (1 - u)
						torsoTwist = -0.26 * (1 - u)
					}
				}
				const idleSway = Math.sin(t * 2.5) * 0.06
				blendBoneRotation(
					rightArm,
					0.98 + slashOffsetX + idleSway,
					0.12,
					0.16 + slashOffsetZ,
					w,
				)
				blendBoneRotation(leftArm, -0.22 - torsoTwist * 0.4, 0, -0.18, w)
				blendBoneRotation(body, -0.05, torsoTwist, 0, w)
				blendBoneRotation(head, 0.04, -0.22 - torsoTwist * 0.5, -0.05, w)
				blendBoneRotation(rightLeg, -0.12, 0, 0.06, w)
				blendBoneRotation(leftLeg, 0.14, 0, -0.06, w)
				break
			}

			case 'solara_hold_pickaxe': {
				// Two satisfying Minecraft mining swings!
				const swing1 = Math.sin(clamp01((t - 0.7) / 1.35) * Math.PI)
				const swing2 = Math.sin(clamp01((t - 2.35) / 1.35) * Math.PI)
				const activeSwing = Math.max(swing1, swing2)
				// Raise up high then strike down cleanly
				const strikeWave =
					t > 0.7 && t < 3.85
						? Math.sin(((t - 0.7) / 1.55) * Math.PI * 2 - Math.PI / 2) * activeSwing
						: 0

				blendBoneRotation(rightArm, 1.12 + strikeWave * 0.85, 0.08, 0.14, w)
				blendBoneRotation(leftArm, 0.18 - strikeWave * 0.25, 0, -0.14, w)
				blendBoneRotation(body, -0.08 + strikeWave * 0.1, 0.08 * strikeWave, 0, w)
				blendBoneRotation(head, -0.08 + strikeWave * 0.12, -0.1, 0, w)
				blendBoneRotation(rightLeg, 0, 0, 0, w)
				blendBoneRotation(leftLeg, 0, 0, 0, w)
				break
			}

			case 'solara_hold_torch': {
				const lookSweep = Math.sin((t - 0.5) * 1.45) * 0.42
				const torchSway = Math.sin(t * 2.2) * 0.05
				blendBoneRotation(rightArm, 1.24 + torchSway, -0.1 + lookSweep * 0.25, 0.2, w)
				blendBoneRotation(leftArm, 0.12, 0, -0.1, w)
				blendBoneRotation(head, 0.06 * Math.sin(t * 1.8), lookSweep, 0.04 * lookSweep, w)
				blendBoneRotation(body, -0.03, lookSweep * 0.2, 0, w)
				blendBoneRotation(rightLeg, 0, 0, 0, w)
				blendBoneRotation(leftLeg, 0, 0, 0, w)

				const torchEntry = this.props.torch
				if (torchEntry) {
					this.updateTorchEmbers(torchEntry, delta, t > 0.2 && t < def.duration - 0.4)
				}
				break
			}

			case 'solara_wave': {
				// Friendly high wave!
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
				// Smooth sit down -> relax & dangle/swing legs + look around -> stand cleanly back up
				const legSwing = Math.sin(t * 3.1) * 0.1
				const lookAround = Math.sin((t - 0.8) * 1.35) * 0.32

				if (orbit) {
					const sitDrop = 0.44 * w
					orbit.position.y = orbitBaseY - sitDrop
				}
				// Legs swing forward into natural seated position
				blendBoneRotation(rightLeg, 1.32 + legSwing, 0, 0.14, w)
				blendBoneRotation(leftLeg, 1.32 - legSwing, 0, -0.14, w)
				// Arms rest relaxed near lap
				blendBoneRotation(rightArm, 0.56, 0, 0.14, w)
				blendBoneRotation(leftArm, 0.56, 0, -0.14, w)
				// Torso slightly leaned back, head looking around casually
				blendBoneRotation(body, 0.08, lookAround * 0.18, 0, w)
				blendBoneRotation(head, 0.05, lookAround, 0.04 * Math.sin(t * 1.5), w)
				break
			}

			case 'solara_cheer_jump': {
				// Anticipation crouch (0..0.45) -> happy jump arc (0.45..1.95) -> soft recovery
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
		}
	}

	cleanup() {
		this.stop()
		for (const entry of Object.values(this.props)) {
			if (entry) disposePropEntry(entry)
		}
		this.props = {}
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

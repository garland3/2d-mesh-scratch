/* tslint:disable */
/* eslint-disable */
export function main(): void;
export class FluidSimulator {
  free(): void;
  constructor(width: number, height: number);
  update(): void;
  add_inlet(x: number, y: number, radius: number): void;
  add_directional_emitter(x: number, y: number, radius: number, angle: number, velocity: number, spread: number): void;
  add_outlet(x: number, y: number, radius: number): void;
  add_wall(x1: number, y1: number, x2: number, y2: number): void;
  clear_environment(): void;
  reset(): void;
  set_gravity(gravity: number): void;
  set_viscosity(viscosity: number): void;
  set_particle_radius(radius: number): void;
  set_stiffness(stiffness: number): void;
  set_rest_density(density: number): void;
  set_max_particles(max: number): void;
  set_time_step(time_step: number): void;
  get_particle_positions(): Float32Array;
  get_particle_velocities(): Float32Array;
  get_vector_field(grid_resolution: number): Float32Array;
  get_particle_count(): number;
  get_particle_radius(): number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_fluidsimulator_free: (a: number, b: number) => void;
  readonly fluidsimulator_new: (a: number, b: number) => number;
  readonly fluidsimulator_update: (a: number) => void;
  readonly fluidsimulator_add_inlet: (a: number, b: number, c: number, d: number) => void;
  readonly fluidsimulator_add_directional_emitter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly fluidsimulator_add_outlet: (a: number, b: number, c: number, d: number) => void;
  readonly fluidsimulator_add_wall: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly fluidsimulator_clear_environment: (a: number) => void;
  readonly fluidsimulator_reset: (a: number) => void;
  readonly fluidsimulator_set_gravity: (a: number, b: number) => void;
  readonly fluidsimulator_set_viscosity: (a: number, b: number) => void;
  readonly fluidsimulator_set_particle_radius: (a: number, b: number) => void;
  readonly fluidsimulator_set_stiffness: (a: number, b: number) => void;
  readonly fluidsimulator_set_rest_density: (a: number, b: number) => void;
  readonly fluidsimulator_set_max_particles: (a: number, b: number) => void;
  readonly fluidsimulator_set_time_step: (a: number, b: number) => void;
  readonly fluidsimulator_get_particle_positions: (a: number) => [number, number];
  readonly fluidsimulator_get_particle_velocities: (a: number) => [number, number];
  readonly fluidsimulator_get_vector_field: (a: number, b: number) => [number, number];
  readonly fluidsimulator_get_particle_count: (a: number) => number;
  readonly fluidsimulator_get_particle_radius: (a: number) => number;
  readonly main: () => void;
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

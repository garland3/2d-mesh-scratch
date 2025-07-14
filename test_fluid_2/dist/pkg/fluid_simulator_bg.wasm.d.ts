/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const __wbg_fluidsimulator_free: (a: number, b: number) => void;
export const fluidsimulator_new: (a: number, b: number) => number;
export const fluidsimulator_update: (a: number) => void;
export const fluidsimulator_add_inlet: (a: number, b: number, c: number, d: number) => void;
export const fluidsimulator_add_directional_emitter: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
export const fluidsimulator_add_outlet: (a: number, b: number, c: number, d: number) => void;
export const fluidsimulator_add_wall: (a: number, b: number, c: number, d: number, e: number) => void;
export const fluidsimulator_clear_environment: (a: number) => void;
export const fluidsimulator_reset: (a: number) => void;
export const fluidsimulator_set_gravity: (a: number, b: number) => void;
export const fluidsimulator_set_viscosity: (a: number, b: number) => void;
export const fluidsimulator_set_particle_radius: (a: number, b: number) => void;
export const fluidsimulator_set_stiffness: (a: number, b: number) => void;
export const fluidsimulator_set_rest_density: (a: number, b: number) => void;
export const fluidsimulator_set_max_particles: (a: number, b: number) => void;
export const fluidsimulator_set_time_step: (a: number, b: number) => void;
export const fluidsimulator_get_particle_positions: (a: number) => [number, number];
export const fluidsimulator_get_particle_velocities: (a: number) => [number, number];
export const fluidsimulator_get_vector_field: (a: number, b: number) => [number, number];
export const fluidsimulator_get_particle_count: (a: number) => number;
export const fluidsimulator_get_particle_radius: (a: number) => number;
export const main: () => void;
export const __wbindgen_export_0: WebAssembly.Table;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_start: () => void;

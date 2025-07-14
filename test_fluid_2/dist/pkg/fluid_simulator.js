let wasm;

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let cachedFloat32ArrayMemory0 = null;

function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

export function main() {
    wasm.main();
}

const FluidSimulatorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fluidsimulator_free(ptr >>> 0, 1));

export class FluidSimulator {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FluidSimulatorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fluidsimulator_free(ptr, 0);
    }
    /**
     * @param {number} width
     * @param {number} height
     */
    constructor(width, height) {
        const ret = wasm.fluidsimulator_new(width, height);
        this.__wbg_ptr = ret >>> 0;
        FluidSimulatorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    update() {
        wasm.fluidsimulator_update(this.__wbg_ptr);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} radius
     */
    add_inlet(x, y, radius) {
        wasm.fluidsimulator_add_inlet(this.__wbg_ptr, x, y, radius);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} radius
     * @param {number} angle
     * @param {number} velocity
     * @param {number} spread
     */
    add_directional_emitter(x, y, radius, angle, velocity, spread) {
        wasm.fluidsimulator_add_directional_emitter(this.__wbg_ptr, x, y, radius, angle, velocity, spread);
    }
    /**
     * @param {number} x
     * @param {number} y
     * @param {number} radius
     */
    add_outlet(x, y, radius) {
        wasm.fluidsimulator_add_outlet(this.__wbg_ptr, x, y, radius);
    }
    /**
     * @param {number} x1
     * @param {number} y1
     * @param {number} x2
     * @param {number} y2
     */
    add_wall(x1, y1, x2, y2) {
        wasm.fluidsimulator_add_wall(this.__wbg_ptr, x1, y1, x2, y2);
    }
    clear_environment() {
        wasm.fluidsimulator_clear_environment(this.__wbg_ptr);
    }
    reset() {
        wasm.fluidsimulator_reset(this.__wbg_ptr);
    }
    /**
     * @param {number} gravity
     */
    set_gravity(gravity) {
        wasm.fluidsimulator_set_gravity(this.__wbg_ptr, gravity);
    }
    /**
     * @param {number} viscosity
     */
    set_viscosity(viscosity) {
        wasm.fluidsimulator_set_viscosity(this.__wbg_ptr, viscosity);
    }
    /**
     * @param {number} radius
     */
    set_particle_radius(radius) {
        wasm.fluidsimulator_set_particle_radius(this.__wbg_ptr, radius);
    }
    /**
     * @param {number} stiffness
     */
    set_stiffness(stiffness) {
        wasm.fluidsimulator_set_stiffness(this.__wbg_ptr, stiffness);
    }
    /**
     * @param {number} density
     */
    set_rest_density(density) {
        wasm.fluidsimulator_set_rest_density(this.__wbg_ptr, density);
    }
    /**
     * @param {number} max
     */
    set_max_particles(max) {
        wasm.fluidsimulator_set_max_particles(this.__wbg_ptr, max);
    }
    /**
     * @param {number} time_step
     */
    set_time_step(time_step) {
        wasm.fluidsimulator_set_time_step(this.__wbg_ptr, time_step);
    }
    /**
     * @returns {Float32Array}
     */
    get_particle_positions() {
        const ret = wasm.fluidsimulator_get_particle_positions(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Float32Array}
     */
    get_particle_velocities() {
        const ret = wasm.fluidsimulator_get_particle_velocities(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {number} grid_resolution
     * @returns {Float32Array}
     */
    get_vector_field(grid_resolution) {
        const ret = wasm.fluidsimulator_get_vector_field(this.__wbg_ptr, grid_resolution);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    get_particle_count() {
        const ret = wasm.fluidsimulator_get_particle_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_particle_radius() {
        const ret = wasm.fluidsimulator_get_particle_radius(this.__wbg_ptr);
        return ret;
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_log_ab081709f2d116fd = function(arg0, arg1) {
        console.log(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_random_3ad904d98382defe = function() {
        const ret = Math.random();
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_0;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedFloat32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('fluid_simulator_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;

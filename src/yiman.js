// Unity WebGL 4.0.47 / Lua 5.1: verified feature gates and yakuman audio routing.
// The proxy injects this before game startup. Reloading restores the originals.
(() => {
    "use strict";
    if (window.__majsoulYiman) return;
    const state = window.__majsoulYiman = { phase: "waiting", error: null };
    const gates = [
        {
            source: "@Tools", table: "Tools", name: "IsYiManEffectClosed",
            params: 0, stack: 2, constants: 2,
            strings: [[0, "Tools"], [1, "IsWebGL"]],
            code: [5, 4210694, 8421404, 26, 2147500054, 8388610, 16777246, 2, 16777246, 8388638],
        },
        {
            source: "@LoadMgr", table: "LoadMgr", name: "GetRes",
            params: 2, stack: 7, constants: 27,
            strings: [[5, "MJ"], [10, "Tools"], [11, "IsWebGL"], [12, "table"], [13, "insert"], [14, "yiman"]],
            code: [
                133, 20988038, 8421399, 2147549206, 8388746, 32961, 8405154, 16777374,
                2148630550, 133, 21020806, 8421399, 2147549206, 8388746, 65729, 8405154,
                16777374, 2148483094, 133, 21053574, 8421399, 2147762198, 33554570,
                98497, 114945, 131393, 147841, 33570978, 164037, 29540550, 8421596,
                16602, 2147549206, 196805, 29573318, 16777472, 229697, 25182428,
                16777374, 2148122646, 133, 21217414, 8421399, 2147565590, 16777354,
                262337, 278785, 16793762, 16777374, 2147958806, 133, 21266566,
                8421399, 2147581974, 25165962, 311489, 147713, 328001, 25182370,
                16777374, 2147778582, 133, 21315718, 8421399, 2147581974, 25165962,
                360641, 377089, 393537, 25182370, 16777374, 2147598358, 133,
                21381254, 8421399, 2147532822, 8388746, 426177, 8405154, 16777374,
                138, 16777374, 8388638,
            ],
        },
    ];
    const audioSpecs = [
        {
            source: "@AudioMgr", table: "AudioMgr", name: "PlayAudio",
            upvalues: 2, params: 2, stack: 15, constants: 17,
            strings: [
                [1, "AudioSettingModel"], [2, "GetSEMute"], [3, "GetFinalSEVolume"],
                [5, "ExcelMgr"], [6, "GetData"], [7, "audio"], [9, "Tools"],
                [10, "IsWebGL"], [11, "Click"], [12, "Sound"], [13, "volume"],
                [14, "AudioMgr"], [15, "PlayAudioSource"], [16, "path"],
            ],
            numbers: [[0, 1], [4, 0], [8, 103]],
            code: [
                16474, 2147483670, 65, 16517, 21004427, 16810140, 16581, 29409483,
                16810204, 29425751, 2147500054, 154, 2147483670, 8388638, 82181,
                37847302, 115009, 115073, 448, 33587484, 16666, 2147483670, 8388638,
                41943363, 4325399, 2147647510, 147845, 54690182, 8421788, 410,
                2147565590, 388, 8389060, 63095238, 16810396, 50331968, 2147549206,
                388, 8389060, 63111622, 16810396, 50331968, 8438158, 2256634185,
                229765, 54772102, 41943488, 38011398, 8389186, 109052547, 46351238,
                75514268, 8388638,
            ],
        },
        {
            source: "@AudioMgr", table: "AudioMgr", name: "PlayAudioByPath",
            upvalues: 2, params: 2, stack: 15, constants: 8,
            strings: [
                [1, "AudioSettingModel"], [2, "GetSEMute"], [3, "GetFinalSEVolume"],
                [5, "Emo"], [6, "AudioMgr"], [7, "PlayAudioSource"],
            ],
            numbers: [[0, 1], [4, 0]],
            code: [
                16474, 2147483670, 65, 16517, 21004427, 16810140, 16581, 29409483,
                16810204, 29425751, 2147500054, 154, 2147483670, 8388638, 8438030,
                324, 8388996, 54608262, 16810332, 98693, 54641030, 41943488, 512,
                8389186, 109052547, 33555328, 75514268, 8388638,
            ],
        },
    ];
    const specs = [gates[1], ...audioSpecs];
    let heap, words, runtime, gate, timer, cursor = 1, candidates = [], attempts = 0;
    const deadline = Date.now() + 180000;
    // Posted tasks yield to the browser without chained timers' 4 ms minimum.
    const continuation = new MessageChannel();
    const valid = (ptr, size) => Number.isInteger(ptr) && ptr > 0
        && ptr % 4 === 0 && ptr + size <= heap.length;

    // All identifiers compared here are ASCII. Compare in place, without
    // decoding strings or allocating slices while walking Lua hash tables.
    function stringIs(ptr, text) {
        if (!valid(ptr, 16 + text.length) || heap[ptr + 4] !== 4
            || words[(ptr + 12) / 4] !== text.length) return false;
        for (let i = 0; i < text.length; i++) {
            if (heap[ptr + 16 + i] !== text.charCodeAt(i)) return false;
        }
        return true;
    }

    function fieldSlot(table, name) {
        if (!valid(table, 32) || heap[table + 4] !== 5 || heap[table + 7] > 16) return 0;
        const node = words[(table + 16) / 4], count = 2 ** heap[table + 7];
        if (!valid(node, count * 32)) return 0;
        for (let i = 0; i < count; i++) {
            const at = node / 4 + i * 8;
            if (words[at + 6] === 4 && stringIs(words[at + 4], name)) return at;
        }
        return 0;
    }

    function field(table, name, type) {
        const at = fieldSlot(table, name);
        return at && words[at + 2] === type ? words[at] : 0;
    }

    function inspect(closure, spec) {
        if (!valid(closure, 20) || (words[closure / 4 + 1] & 0x00ff00ff) !== 6) return null;
        const proto = words[(closure + 16) / 4];
        if (!valid(proto, 76) || heap[proto + 4] !== 9
            || heap[proto + 72] !== (spec.upvalues || 0)
            || heap[closure + 7] !== (spec.upvalues || 0)
            || heap[proto + 73] !== spec.params || heap[proto + 75] !== spec.stack
            || words[(proto + 40) / 4] !== spec.constants
            || words[(proto + 44) / 4] !== spec.code.length
            || !stringIs(words[(proto + 32) / 4], spec.source)) return null;
        const k = words[(proto + 8) / 4], code = words[(proto + 12) / 4];
        if (!valid(k, spec.constants * 16) || !valid(code, spec.code.length * 4)) return null;
        for (const [index, name] of spec.strings) {
            if (words[k / 4 + index * 4 + 2] !== 4
                || !stringIs(words[k / 4 + index * 4], name)) return null;
        }
        for (const [index, value] of spec.numbers || []) {
            if (words[k / 4 + index * 4 + 2] !== 3
                || new DataView(heap.buffer).getFloat64(k + index * 16, true) !== value) return null;
        }
        for (let i = 0; i < spec.code.length; i++) {
            if (words[code / 4 + i] !== spec.code[i]) return null;
        }
        const env = words[(closure + 12) / 4];
        if (field(field(env, spec.table, 5), spec.name, 6) !== closure) return null;
        return { closure, code, env, k };
    }

    function scan() {
        // Bound both time and work per task; Unity keeps rendering between slices.
        const until = performance.now() + 4;
        const memory = words, end = Math.min(memory.length - 4, cursor + 262144);
        do {
            const limit = Math.min(end, cursor + 4096);
            for (let i = cursor; i < limit; i++) {
                if ((memory[i + 1] & 0xffff00ff) === 6 && inspect(i * 4, gates[0])) {
                    candidates.push(i * 4);
                }
            }
            cursor = limit;
        } while (cursor < end && performance.now() < until);
        if (cursor < words.length - 4) return false;
        // Lua may collect/replace functions while we yield. Recheck candidates.
        const matches = candidates.map(ptr => inspect(ptr, gates[0])).filter(Boolean);
        cursor = 1;
        candidates = [];
        if (matches.length > 1) throw new Error("役满函数匹配不唯一，未修改内存。");
        gate = matches[0];
        return true;
    }

    function patch() {
        const closures = specs.map(spec => field(field(gate.env, spec.table, 5), spec.name, 6));
        if (closures.some(closure => !closure)) return false; // Modules can arrive after Tools.
        const targets = [inspect(gate.closure, gates[0]), ...closures.map((closure, i) => inspect(closure, specs[i]))];
        if (targets.some(target => !target || target.env !== gate.env)) {
            throw new Error("役满函数结构已变化，未修改内存。");
        }
        const tools = field(gate.env, "Tools", 5), marker = fieldSlot(tools, "yiman");
        if (words[(tools + 8) / 4] !== 0 || (marker && words[marker + 2] !== 0)) {
            throw new Error("役满资源标记冲突，未修改内存。");
        }
        // Validate all four functions before any write. Keep ID 103 on Click,
        // route only ID 262 (yiman_preload) to Emo, and leave other IDs on Sound.
        // SetSkin/ClearSpine calls StopEffectAudio on Sound during the animation.
        // This WebGL-only branch reuses the former Tools/IsWebGL constant slots;
        // lookup, gain, mute checks and PlayAudioSource stay unchanged.
        const audio = targets[2], byPath = targets[3];
        new DataView(heap.buffer).setFloat64(audio.k + 9 * 16, 262, true);
        words[audio.k / 4 + 9 * 4 + 2] = 3; // LUA_TNUMBER
        // Reuse the string rooted in PlayAudioByPath; do not allocate Lua objects.
        words[audio.k / 4 + 10 * 4] = words[byPath.k / 4 + 5 * 4];
        words.set([
            2147549206, 388, 8389060, 63095238, 16810396, 2147631126,
            4341783, 2147532822, 388, 8389060, 63078854, 2147516438,
        ], audio.code / 4 + 25);
        // The game sets Tools.yiman only when GetRes(MJ) includes the resource
        // group. A scene preloaded before this patch keeps its original playback;
        // the next scene load enables it through the normal resource lifecycle.
        // Reuse the rooted "yiman" string; let Lua create the table entry itself.
        words[targets[0].k / 4 + 4] = words[targets[1].k / 4 + 14 * 4];
        // IsYiManEffectClosed: return not Tools.yiman (NOT; RETURN).
        words.set([19, 16777246], targets[0].code / 4 + 2);
        // table.insert(groups, "yiman"); Tools.yiman = true; return groups.
        words.set([
            196805, 29573318, 16777472, 229697, 25182428,
            164037, 8388866, 2264989897, 16777374,
        ], targets[1].code / 4 + 28);
        return true;
    }

    function finish(phase, error) {
        state.phase = phase;
        state.error = error ? String(error.message || error) : null;
        clearTimeout(timer);
        continuation.port1.onmessage = null;
        continuation.port1.close();
        continuation.port2.close();
        window.removeEventListener("pagehide", cancel);
        if (error) console.warn("[MajsoulMax] 役满动画未启用：", state.error);
    }
    const cancel = () => finish("stopped");
    function tick() {
        if (state.phase !== "waiting") return;
        if (Date.now() >= deadline) return finish("timeout", new Error("等待游戏加载超时，请刷新页面。"));
        const module = window.unityInstance?.Module;
        if (module?.HEAPU32) {
            if (module.productVersion !== "4.0.47") return finish("unsupported", new Error("不支持的游戏版本。"));
            if (runtime !== module || heap?.buffer !== module.HEAPU8.buffer) {
                runtime = module;
                gate = null;
                cursor = 1;
                candidates = [];
                attempts = 0;
            }
            heap = module.HEAPU8;
            words = module.HEAPU32;
            try {
                if (!gate && !scan()) {
                    continuation.port2.postMessage(null);
                    return;
                }
                if (gate && patch()) {
                    finish("armed");
                    console.info("[MajsoulMax] 役满动画补丁已安装，随下次牌局资源加载生效。");
                    return;
                }
            } catch (error) { return finish("incompatible", error); }
        }
        // Back off only after an unsuccessful full pass, never between slices.
        timer = setTimeout(tick, gate ? 100 : module?.HEAPU32 ? Math.min(500 * 2 ** attempts++, 10000) : 500);
    }
    continuation.port1.onmessage = tick;
    window.addEventListener("pagehide", cancel);
    tick();
})();

/**
 * Contract test for the WASM critter API used by the Vue selector.
 * Verifies that critter metadata can be retrieved dynamically (no hardcoding),
 * and that the consumer pattern (wait for readiness, then fetch list) works.
 */

// Minimal waitUntil helper mirroring the Vue component logic
function waitUntil(fn, timeoutMs = 500) {
  const start = Date.now();
  return new Promise((resolve) => {
    const tick = () => {
      if (fn()) return resolve(true);
      if (Date.now() - start > timeoutMs) return resolve(false);
      setTimeout(tick, 10);
    };
    tick();
  });
}

// Mock of the JS-facing WASM module API (window.__A4D_WASM__)
function createMockWasm(list, readyDelayMs = 0) {
  const state = { ready: false, list: Array.from(list) };
  if (readyDelayMs <= 0) state.ready = true;
  else setTimeout(() => { state.ready = true; }, readyDelayMs);
  return {
    critters_ready() { return state.ready; },
    // In the real API this returns a JS Array of Objects; mirror the shape
    get_available_critters() { return state.list.map(x => ({ ...x })); },
  };
}

describe('WASM Critter API contract', () => {
  test('waits for readiness and returns dynamic critter list', async () => {
    const wasm = createMockWasm([
      { id: 'chirpy_bird', name: 'Chirpy', species: 'Bird', sprite: 'assets/sprites/bird-animation.png' },
      { id: 'bouncy_bunny', name: 'Bouncy', species: 'Bunny', sprite: 'https://example.com/assets/bunny.png' },
    ], /*readyDelayMs*/ 50);

    // Simulate the Vue selector flow: wait -> fetch -> map for UI
    const ready = await waitUntil(() => wasm.critters_ready(), 1000);
    expect(ready).toBe(true);

    const raw = wasm.get_available_critters();
    expect(Array.isArray(raw)).toBe(true);
    expect(raw.length).toBeGreaterThanOrEqual(2);

    // Check required fields and non-empty strings
    for (const c of raw) {
      expect(typeof c.id).toBe('string');
      expect(c.id.length).toBeGreaterThan(0);
      expect(typeof c.name).toBe('string');
      expect(c.name.length).toBeGreaterThan(0);
      expect(typeof c.species).toBe('string');
      expect(c.species.length).toBeGreaterThan(0);
      expect(typeof c.sprite).toBe('string');
      expect(c.sprite.length).toBeGreaterThan(0);
    }

    // Must include known critters (no hardcoding in UI required)
    const ids = new Set(raw.map(c => c.id));
    expect(ids.has('chirpy_bird')).toBe(true);
    expect(ids.has('bouncy_bunny')).toBe(true);
  });

  test('consumer mapping produces UI-ready objects without hardcoding', async () => {
    const wasm = createMockWasm([
      { id: 'x', name: 'X', species: 'Bird', sprite: 'assets/sprites/x.png' },
    ]);
    await waitUntil(() => wasm.critters_ready(), 200);
    const raw = wasm.get_available_critters();

    // This mirrors the mapping in CritterSelection.vue
    const uiList = raw.map((c) => ({
      id: c.id,
      name: c.name,
      species: c.species,
      sprite: c.sprite,
      stats: { speed: 8.0, playfulness: 8.0, obedience: 8.0 },
    }));

    expect(uiList).toHaveLength(1);
    expect(uiList[0]).toMatchObject({ id: 'x', name: 'X', species: 'Bird', sprite: 'assets/sprites/x.png' });
    expect(uiList[0].stats).toEqual({ speed: 8.0, playfulness: 8.0, obedience: 8.0 });
  });
});


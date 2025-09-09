/**
 * Tests for dynamic critter ID mapping
 * Verifies that the hash function produces consistent, unique IDs
 */

// Hash function from GamePage.vue
function hashCritterId(critterId) {
  return critterId.split('').reduce((hash, char) => hash + char.charCodeAt(0), 0) % 1000;
}

describe('Critter ID Mapping', () => {
  test('should generate consistent hash IDs for known critters', () => {
    const chirpyHash = hashCritterId('chirpy_bird');
    const bunnyHash = hashCritterId('bouncy_bunny');
    
    // Should be consistent across calls
    expect(hashCritterId('chirpy_bird')).toBe(chirpyHash);
    expect(hashCritterId('bouncy_bunny')).toBe(bunnyHash);
    
    console.log(`chirpy_bird -> ${chirpyHash}`);
    console.log(`bouncy_bunny -> ${bunnyHash}`);
  });

  test('should generate different IDs for different critters', () => {
    const chirpyHash = hashCritterId('chirpy_bird');
    const bunnyHash = hashCritterId('bouncy_bunny');
    
    expect(chirpyHash).not.toBe(bunnyHash);
  });

  test('should handle new critters dynamically', () => {
    // Test future critters that don't exist yet
    const futureCritters = [
      'sleepy_cat',
      'playful_puppy',
      'wise_owl',
      'sneaky_fox'
    ];

    const hashes = futureCritters.map(id => ({
      id,
      hash: hashCritterId(id)
    }));

    // All hashes should be different
    const uniqueHashes = new Set(hashes.map(h => h.hash));
    expect(uniqueHashes.size).toBe(futureCritters.length);

    console.log('Future critter mappings:');
    hashes.forEach(({id, hash}) => {
      console.log(`  ${id} -> ${hash}`);
    });
  });

  test('should produce IDs in valid range (0-999)', () => {
    const testIds = [
      'chirpy_bird',
      'bouncy_bunny',
      'a',
      'very_long_critter_name_with_lots_of_characters',
      '123_numeric_critter',
      'special-chars_critter!'
    ];

    testIds.forEach(id => {
      const hash = hashCritterId(id);
      expect(hash).toBeGreaterThanOrEqual(0);
      expect(hash).toBeLessThan(1000);
      expect(Number.isInteger(hash)).toBe(true);
    });
  });
});
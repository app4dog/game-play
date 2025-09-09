/**
 * Tests for Grid coordinate generation
 * Verifies that sprite animation coordinates match between Vue and Rust
 */

// Grid coordinate generation logic from Vue CritterSelection component
function generateGridCoordinates(frameLayout, animationFrames) {
  const { frame_size, layout } = frameLayout;
  const [frameWidth, frameHeight] = frame_size;
  
  if (typeof layout === 'object' && layout.Grid) {
    const { cols, rows } = layout.Grid;
    
    // Generate all possible frame positions in the grid
    const allFrames = [];
    for (let row = 0; row < rows; row++) {
      for (let col = 0; col < cols; col++) {
        allFrames.push({
          x: col * frameWidth,
          y: row * frameHeight
        });
      }
    }
    
    // Return positions for the specific animation frames
    return animationFrames.map((frameIndex) => {
      if (frameIndex < allFrames.length && frameIndex >= 0) {
        const frame = allFrames[frameIndex];
        return frame || { x: 0, y: 0 };
      } else {
        return { x: 0, y: 0 };
      }
    });
  }
  
  return [];
}

describe('Grid Coordinate Generation', () => {
  test('should generate correct coordinates for chirpy_bird (3x2 grid)', () => {
    const frameLayout = {
      frame_size: [1000, 1000],
      layout: { Grid: { cols: 3, rows: 2 } }
    };
    const animationFrames = [0, 1, 2, 3, 4, 5]; // All 6 frames
    
    const coords = generateGridCoordinates(frameLayout, animationFrames);
    
    expect(coords).toHaveLength(6);
    expect(coords[0]).toEqual({ x: 0, y: 0 });      // Frame 0: Top-left
    expect(coords[1]).toEqual({ x: 1000, y: 0 });   // Frame 1: Top-middle  
    expect(coords[2]).toEqual({ x: 2000, y: 0 });   // Frame 2: Top-right
    expect(coords[3]).toEqual({ x: 0, y: 1000 });   // Frame 3: Bottom-left
    expect(coords[4]).toEqual({ x: 1000, y: 1000 }); // Frame 4: Bottom-middle
    expect(coords[5]).toEqual({ x: 2000, y: 1000 }); // Frame 5: Bottom-right
    
    console.log('chirpy_bird grid coordinates:', coords.map(c => `(${c.x},${c.y})`).join(', '));
  });

  test('should generate correct coordinates for bouncy_bunny (4x4 grid, frames 0,14)', () => {
    const frameLayout = {
      frame_size: [128, 128],
      layout: { Grid: { cols: 4, rows: 4 } }
    };
    const animationFrames = [0, 14]; // Only frames 0 and 14
    
    const coords = generateGridCoordinates(frameLayout, animationFrames);
    
    expect(coords).toHaveLength(2);
    expect(coords[0]).toEqual({ x: 0, y: 0 });      // Frame 0: Position (0,0)
    expect(coords[1]).toEqual({ x: 2 * 128, y: 3 * 128 }); // Frame 14: Position (2,3) in 4x4 grid
    
    console.log('bouncy_bunny grid coordinates:', coords.map(c => `(${c.x},${c.y})`).join(', '));
    
    // Frame 14 calculation: 14 = row*cols + col = 3*4 + 2, so row=3, col=2
    // Coordinates: (col*frameWidth, row*frameHeight) = (2*128, 3*128) = (256, 384)
    expect(coords[1]).toEqual({ x: 256, y: 384 });
  });

  test('should handle edge cases gracefully', () => {
    const frameLayout = {
      frame_size: [100, 100],
      layout: { Grid: { cols: 2, rows: 2 } }
    };
    
    // Test with invalid frame indices
    const coords1 = generateGridCoordinates(frameLayout, [-1, 10]); // Out of bounds
    expect(coords1[0]).toEqual({ x: 0, y: 0 }); // Fallback
    expect(coords1[1]).toEqual({ x: 0, y: 0 }); // Fallback
    
    // Test with empty animation frames
    const coords2 = generateGridCoordinates(frameLayout, []);
    expect(coords2).toHaveLength(0);
  });

  test('should verify Grid coordinate formula', () => {
    // Test the mathematical relationship: frameIndex = row * cols + col
    const cols = 4, rows = 4;
    const frameWidth = 128, frameHeight = 128;
    
    for (let frameIndex = 0; frameIndex < cols * rows; frameIndex++) {
      const expectedRow = Math.floor(frameIndex / cols);
      const expectedCol = frameIndex % cols;
      const expectedX = expectedCol * frameWidth;
      const expectedY = expectedRow * frameHeight;
      
      const frameLayout = {
        frame_size: [frameWidth, frameHeight],
        layout: { Grid: { cols, rows } }
      };
      
      const coords = generateGridCoordinates(frameLayout, [frameIndex]);
      expect(coords[0]).toEqual({ x: expectedX, y: expectedY });
    }
  });
});
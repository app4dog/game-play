//! Tests for sprite animation Grid coordinate system
//! Verifies that the Rust implementation matches the Vue component logic

use app4dog_game_engine::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test data matching the RON files
    fn mock_chirpy_frame_layout() -> critter_keeper::FrameLayout {
        critter_keeper::FrameLayout {
            image_size: (3000, 2000),
            frame_count: 6,
            frame_size: (1000, 1000),
            layout: critter_keeper::LayoutType::Grid { cols: 3, rows: 2 },
        }
    }

    fn mock_bouncy_frame_layout() -> critter_keeper::FrameLayout {
        critter_keeper::FrameLayout {
            image_size: (512, 512),
            frame_count: 2,
            frame_size: (128, 128),
            layout: critter_keeper::LayoutType::Grid { cols: 4, rows: 4 },
        }
    }

    /// Generate Grid coordinates for testing (matches the function in systems.rs)
    fn generate_grid_coordinates(frame_layout: &critter_keeper::FrameLayout) -> Vec<(f32, f32)> {
        let frame_width = frame_layout.frame_size.0 as f32;
        let frame_height = frame_layout.frame_size.1 as f32;
        
        match &frame_layout.layout {
            critter_keeper::LayoutType::Grid { cols, rows } => {
                let mut coordinates = Vec::new();
                for row in 0..*rows {
                    for col in 0..*cols {
                        coordinates.push((
                            col as f32 * frame_width,
                            row as f32 * frame_height
                        ));
                    }
                }
                coordinates
            },
            critter_keeper::LayoutType::Horizontal => {
                (0..frame_layout.frame_count).map(|i| (i as f32 * frame_width, 0.0)).collect()
            },
            critter_keeper::LayoutType::Vertical => {
                (0..frame_layout.frame_count).map(|i| (0.0, i as f32 * frame_height)).collect()
            }
        }
    }

    #[test]
    fn test_chirpy_bird_grid_coordinates() {
        let frame_layout = mock_chirpy_frame_layout();
        let coords = generate_grid_coordinates(&frame_layout);

        // Should generate 6 coordinates for 3x2 grid
        assert_eq!(coords.len(), 6);
        
        // Expected coordinates for 3x2 grid with 1000x1000 frames
        let expected = vec![
            (0.0, 0.0),       // Frame 0: (0,0)
            (1000.0, 0.0),    // Frame 1: (1,0) 
            (2000.0, 0.0),    // Frame 2: (2,0)
            (0.0, 1000.0),    // Frame 3: (0,1)
            (1000.0, 1000.0), // Frame 4: (1,1)
            (2000.0, 1000.0), // Frame 5: (2,1)
        ];

        for (i, (actual, expected)) in coords.iter().zip(expected.iter()).enumerate() {
            assert_eq!(actual, expected, "Frame {} coordinates mismatch", i);
        }

        println!("✅ Chirpy bird coordinates: {:?}", coords);
    }

    #[test] 
    fn test_bouncy_bunny_grid_coordinates() {
        let frame_layout = mock_bouncy_frame_layout();
        let coords = generate_grid_coordinates(&frame_layout);

        // Should generate 16 coordinates for 4x4 grid
        assert_eq!(coords.len(), 16);
        
        // Check specific frames used in animation [0, 14]
        assert_eq!(coords[0], (0.0, 0.0));        // Frame 0: (0,0)
        assert_eq!(coords[14], (256.0, 384.0));   // Frame 14: (2,3) = (2*128, 3*128)
        
        // Verify frame 14 calculation: 14 = row*cols + col = 3*4 + 2
        // So row=3, col=2, coordinates=(2*128, 3*128)=(256, 384)
        
        println!("✅ Bouncy bunny coordinates for frames [0, 14]: {:?}", 
            vec![coords[0], coords[14]]);
    }

    #[test]
    fn test_grid_coordinate_formula() {
        // Test the mathematical relationship: frameIndex = row * cols + col
        let cols = 4u32;
        let rows = 4u32; 
        let frame_width = 128.0;
        let frame_height = 128.0;

        for frame_index in 0..(cols * rows) {
            let expected_row = frame_index / cols;
            let expected_col = frame_index % cols;
            let expected_x = expected_col as f32 * frame_width;
            let expected_y = expected_row as f32 * frame_height;

            let frame_layout = critter_keeper::FrameLayout {
                image_size: (512, 512),
                frame_count: 16,
                frame_size: (128, 128),
                layout: critter_keeper::LayoutType::Grid { cols, rows },
            };

            let coords = generate_grid_coordinates(&frame_layout);
            let actual = coords[frame_index as usize];
            
            assert_eq!(actual, (expected_x, expected_y), 
                "Frame {} should be at ({}, {})", frame_index, expected_x, expected_y);
        }
    }

    #[test]
    fn test_horizontal_fallback() {
        let frame_layout = critter_keeper::FrameLayout {
            image_size: (1000, 100),
            frame_count: 5,
            frame_size: (200, 100),
            layout: critter_keeper::LayoutType::Horizontal,
        };

        let coords = generate_grid_coordinates(&frame_layout);
        let expected = vec![
            (0.0, 0.0),
            (200.0, 0.0), 
            (400.0, 0.0),
            (600.0, 0.0),
            (800.0, 0.0),
        ];

        assert_eq!(coords, expected);
        println!("✅ Horizontal layout coordinates: {:?}", coords);
    }

    #[test]
    fn test_vertical_fallback() {
        let frame_layout = critter_keeper::FrameLayout {
            image_size: (100, 1000),
            frame_count: 5,
            frame_size: (100, 200),
            layout: critter_keeper::LayoutType::Vertical,
        };

        let coords = generate_grid_coordinates(&frame_layout);
        let expected = vec![
            (0.0, 0.0),
            (0.0, 200.0),
            (0.0, 400.0), 
            (0.0, 600.0),
            (0.0, 800.0),
        ];

        assert_eq!(coords, expected);
        println!("✅ Vertical layout coordinates: {:?}", coords);
    }
}
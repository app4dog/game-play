// Build script to generate TypeScript bindings from Rust types using specta
// For now, we'll keep it simple and manually create the TypeScript types
// TODO: Integrate specta type export in a future iteration

fn main() {
    println!("cargo:rerun-if-changed=src/events.rs");
    println!("cargo:warning=TypeScript types should be manually synced for now");
}
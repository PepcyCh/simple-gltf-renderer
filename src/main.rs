use anyhow::*;

mod camera;
mod engine;
mod gltf_scene;
mod graphics;
mod light;
mod material;
mod mesh;
mod shader;
mod texture;
mod render_target;
mod vertex;

fn main() -> Result<()> {
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() != 3 {
    //     println!("Usage cargo run <path-to-gltf> <path-to-shader-json>");
    //     return Ok(());
    // }

    println!("Creating engine...");
    let (mut engine, event_loop) = engine::Engine::new()?;
    println!("Engine is created successfully. Loading shaders...");
    // engine.load_shaders(&args[2])?;
    engine.load_shaders("res/models/radio-gltf/shaders.json")?;
    // engine.load_shaders("res/models/handley_page_hp42-gltf/shaders.json")?;
    println!("Shaders are loaded successfully. Loading glTF scene...");
    // engine.load_gltf(&args[1])?;
    engine.load_gltf("res/models/radio-gltf/scene.gltf")?;
    // engine.load_gltf("res/models/handley_page_hp42-gltf/scene.gltf")?;
    println!("glTF scene is loaded successfully. Running...");
    engine.run(event_loop);

    Ok(())
}

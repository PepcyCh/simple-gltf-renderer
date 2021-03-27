use anyhow::*;

mod camera;
mod engine;
mod env_map;
mod gltf_scene;
mod graphics;
mod inner_pipelines;
mod light;
mod material;
mod mesh;
mod shader;
mod texture;
mod vertex;

fn main() -> Result<()> {
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() != 3 {
    //     println!("Usage cargo run <path-to-gltf> <path-to-shader-json>");
    //     return Ok(());
    // }

    println!("Creating engine...");
    let (mut engine, event_loop) = engine::Engine::new()?;

    println!("Engine is created successfully. Loading skybox...");
    // TODO - there is some problems about skybox but I don't know why...
    // the codes below will make program crash when later 'engine.generate_all_mipmaps();' is executed
    // but if I just use the default 1x1x6 skybox, there will be no problem
    engine.load_skybox(
        "res/textures/skybox-sea/front.jpg",
        "res/textures/skybox-sea/back.jpg",
        "res/textures/skybox-sea/top.jpg",
        "res/textures/skybox-sea/bottom.jpg",
        "res/textures/skybox-sea/right.jpg",
        "res/textures/skybox-sea/left.jpg",
    );

    println!("Skybox is loaded successfully. Loading shaders...");
    // engine.load_shaders(&args[2])?;
    engine.load_shaders("res/models/radio-gltf/shaders.json")?;
    // engine.load_shaders("res/models/handley_page_hp42-gltf/shaders.json")?;

    println!("Shaders are loaded successfully. Loading glTF scene...");
    // engine.load_gltf(&args[1])?;
    engine.load_gltf("res/models/radio-gltf/scene.gltf")?;
    // engine.load_gltf("res/models/handley_page_hp42-gltf/scene.gltf")?;

    println!("glTF scene is loaded successfully. Generating mipmaps...");
    engine.generate_all_mipmaps();

    println!("Mipmaps are generated successfully. Running...");
    engine.run(event_loop);

    Ok(())
}

/*
    Copyright 2024 notskm

    This file is part of garlicjr.

    garlicjr is free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by the Free
    Software Foundation, either version 3 of the License, or (at your option)
    any later version.

    garlicjr is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
    more details.

    You should have received a copy of the GNU General Public License along
    with garlicjr. If not, see <https: //www.gnu.org/licenses/>.
*/

use glow::HasContext;
use imgui::{Context, Drag};
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
use sdl2::{
    event::Event,
    video::{GLProfile, Window},
    Sdl, VideoSubsystem,
};

use garlicjr::{Bus, ReadWriteMode, SharpSM83};

// Create a new glow context.
fn glow_context(window: &Window) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    }
}

fn init_sdl() -> (Sdl, VideoSubsystem) {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    (sdl, video_subsystem)
}

fn main() {
    /* initialize SDL and its video subsystem */
    let (sdl, video_subsystem) = init_sdl();

    /* hint SDL to initialize an OpenGL 3.3 core profile context */
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    /* create a new window, be sure to call opengl method on the builder when using glow! */
    let window = video_subsystem
        .window("GarlicJr", 1280, 720)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    /* create a new OpenGL context and make it current */
    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();

    /* enable vsync to cap framerate */
    window.subsystem().gl_set_swap_interval(1).unwrap();

    /* create new glow and imgui contexts */
    let gl = glow_context(&window);

    /* create context */
    let mut imgui = Context::create();

    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    /* create platform and renderer */
    let mut platform = SdlPlatform::init(&mut imgui);
    let mut renderer = AutoRenderer::initialize(gl, &mut imgui).unwrap();

    /* start main loop */
    let mut event_pump = sdl.event_pump().unwrap();

    let mut cpu = SharpSM83::new();
    let mut bus = Bus::new();

    'main: loop {
        for event in event_pump.poll_iter() {
            /* pass all events to imgui platfrom */
            platform.handle_event(&mut imgui, &event);

            if let Event::Quit { .. } = event {
                break 'main;
            }
        }

        /* call prepare_frame before calling imgui.new_frame() */
        platform.prepare_frame(&mut imgui, &window, &event_pump);

        let ui = imgui.new_frame();

        /* create imgui UI here */
        cpu_window(ui, &mut cpu, &mut bus);
        bus_window(ui, &mut bus);

        /* render */
        let draw_data = imgui.render();

        unsafe { renderer.gl_context().clear(glow::COLOR_BUFFER_BIT) };
        renderer.render(draw_data).unwrap();

        window.gl_swap_window();
    }
}

fn cpu_window(ui: &imgui::Ui, cpu: &mut SharpSM83, bus: &mut Bus) {
    ui.window("CPU")
        .size([200.0, 200.0], imgui::Condition::FirstUseEver)
        .build(|| input_cpu(ui, cpu, bus));
}

fn input_cpu(ui: &imgui::Ui, cpu: &mut SharpSM83, bus: &mut Bus) {
    let mut af = [cpu.registers.a, cpu.registers.f];
    let mut bc = [cpu.registers.b, cpu.registers.c];
    let mut de = [cpu.registers.d, cpu.registers.e];
    let mut hl = [cpu.registers.h, cpu.registers.l];

    Drag::new("AF").build_array(ui, &mut af);
    Drag::new("BC").build_array(ui, &mut bc);
    Drag::new("DE").build_array(ui, &mut de);
    Drag::new("HL").build_array(ui, &mut hl);
    Drag::new("SP").build(ui, &mut cpu.registers.stack_pointer);
    Drag::new("PC").build(ui, &mut cpu.registers.program_counter);

    cpu.registers.a = af[0];
    cpu.registers.f = af[1];
    cpu.registers.b = bc[0];
    cpu.registers.c = bc[1];
    cpu.registers.d = de[0];
    cpu.registers.e = de[1];
    cpu.registers.h = hl[0];
    cpu.registers.l = hl[1];

    if ui.button("Step") {
        cpu.tick(bus);
    }
}

fn bus_window(ui: &imgui::Ui, bus: &mut Bus) {
    ui.window("Bus")
        .size([200.0, 200.0], imgui::Condition::FirstUseEver)
        .build(|| input_bus(ui, bus));
}

fn input_bus(ui: &imgui::Ui, bus: &mut Bus) {
    ui.input_scalar("address", &mut bus.address).build();
    ui.input_scalar("data", &mut bus.data).build();
    ui.radio_button("read", &mut bus.mode, ReadWriteMode::Read);
    ui.radio_button("write", &mut bus.mode, ReadWriteMode::Write);
}

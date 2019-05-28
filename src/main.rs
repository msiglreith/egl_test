extern crate glutin_egl_sys as egl_sys;

use std::ffi;
use std::ptr;

use grr::Object;

const VERTEX_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec2 v_pos;
    layout (location = 1) in vec3 v_color;

    layout (location = 0) out vec3 a_color;

    void main() {
        a_color = v_color;
        gl_Position = vec4(v_pos, 0.0, 1.0);
    }
"#;

const FRAGMENT_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec3 a_color;
    out vec4 f_color;

    void main() {
       f_color = vec4(a_color, 1.0);
    }
"#;

const VERTICES: [f32; 15] = [
    -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 0.5, 0.0, 0.0, 1.0,
];

use winit::os::unix::WindowExt;

fn main() -> grr::Result<()> {
    unsafe {
        let mut events_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new()
            .with_title("Hello, world!")
            .with_dimensions(winit::dpi::LogicalSize {
                width: 1024.0,
                height: 768.0,
            })
            .build(&events_loop)
            .unwrap();

        let lib = libloading::Library::new("libEGL.so").unwrap();

        let egl = egl_sys::egl::Egl::load_with(|sym| unsafe {
            lib.get(
                ffi::CString::new(sym.as_bytes())
                    .unwrap()
                    .as_bytes_with_nul(),
            )
            .map(|sym| *sym)
            .unwrap_or(std::ptr::null_mut())
        });

        let display = egl.GetDisplay(egl_sys::egl::DEFAULT_DISPLAY);

        let mut major = 0;
        let mut minor = 0;

        dbg!(egl.Initialize(display, &mut major, &mut minor));
        dbg!((major, minor));

        dbg!(egl.BindAPI(egl_sys::egl::OPENGL_API));
        let extensions = dbg!(egl.QueryString(display, egl_sys::egl::EXTENSIONS as _));
        println!("{:?}", ffi::CStr::from_ptr(extensions));

        let mut config = ptr::null();
        let mut num_configs = 0;
        let attribs = [
            egl_sys::egl::SURFACE_TYPE,
            egl_sys::egl::WINDOW_BIT,
            egl_sys::egl::NONE,
        ];
        egl.ChooseConfig(
            display,
            attribs.as_ptr() as *const _,
            &mut config as *mut _ as *mut _,
            1,
            &mut num_configs,
        );

        let attribs = [
            egl_sys::egl::CONTEXT_MAJOR_VERSION,
            4,
            egl_sys::egl::CONTEXT_MINOR_VERSION,
            5,
            egl_sys::egl::NONE,
        ];
        let context = dbg!(egl.CreateContext(
            display,
            config,
            egl_sys::egl::NO_CONTEXT,
            attribs.as_ptr() as *const _
        ));

        let surface = window.get_xlib_window().unwrap();

        let attribs = [
            egl_sys::egl::WIDTH,
            1024,
            egl_sys::egl::HEIGHT,
            768,
            egl_sys::egl::NONE,
        ];
        dbg!(egl.GetError());
        let egl_surface =
            dbg!(egl.CreateWindowSurface(display, config, dbg!(surface) as _, ptr::null(),));
        dbg!(egl.GetError());

        dbg!(egl.MakeCurrent(display, egl_surface, egl_surface, context));

        let grr = grr::Device::new(
            |symbol| egl.GetProcAddress(symbol.as_ptr() as _) as *const _,
            grr::Debug::Enable {
                callback: |_, _, _, _, msg| {
                    println!("{:?}", msg);
                },
                flags: grr::DebugReport::FULL,
            },
        );

        let texture = grr.create_image(
            grr::ImageType::D2 {
                width: 100,
                height: 100,
                layers: 1,
                samples: 1,
            },
            grr::Format::R8G8B8A8_SRGB,
            1,
        )?;

        dbg!(egl.GetError());

        dbg!(egl.CreateImage(
            display,
            context,
            egl_sys::egl::GL_TEXTURE_2D,
            dbg!(texture.handle()) as _,
            ptr::null(),
        ));

        dbg!(egl.GetError());
    }

    Ok(())
}

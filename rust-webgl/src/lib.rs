use js_sys::WebAssembly;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, KeyboardEvent, MouseEvent, WebGlBuffer, WebGlProgram, WebGlRenderingContext,
    WebGlShader, WebGlUniformLocation,
};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct App {
    canvas: HtmlCanvasElement,
    context: WebGlRenderingContext,
    program: WebGlProgram,
    buffer: WebGlBuffer,
    translation_uniform_location: WebGlUniformLocation,
    translation: RefCell<[f32; 2]>,
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> App {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().unwrap();

        let context = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()
            .unwrap();

        let vert_shader = compile_shader(
            &context,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"
            attribute vec4 position;
            uniform vec2 translation;
            void main() {
                gl_Position = vec4(
                    position.x - translation.x,
                    position.y - translation.y,
                    position.z,
                    position.w);
            }
        "#,
        )
        .unwrap();
        let frag_shader = compile_shader(
            &context,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"
            void main() {
                gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
            }
        "#,
        )
        .unwrap();
        let program = link_program(&context, [vert_shader, frag_shader].iter()).unwrap();

        let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let vertices_location = vertices.as_ptr() as u32 / 4;
        let vert_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(vertices_location, vertices_location + vertices.len() as u32);

        let buffer = context
            .create_buffer()
            .ok_or("failed to create buffer")
            .unwrap();
        context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGlRenderingContext::STATIC_DRAW,
        );

        let translation_uniform_location = context
            .get_uniform_location(&program, "translation")
            .unwrap();

        App {
            canvas,
            context,
            program,
            buffer,
            translation_uniform_location,
            translation: RefCell::new([0.0, 0.0]),
        }
    }

    fn on_mouse_down(&self, event: MouseEvent) {
        log(&format!("x: {} y: {}", event.offset_x(), event.offset_y()));
        *self.translation.borrow_mut() = [
            -event.offset_x() as f32 / 150.0,
            event.offset_y() as f32 / 150.0,
        ];
        self.draw();
    }

    fn on_key_down(&self, event: KeyboardEvent) {
        log(&format!("keydown: {:?}", event.key()));
        {
            let mut translation = self.translation.borrow_mut();
            match event.key().as_str() {
                "ArrowUp" => translation[1] -= 0.1,
                "ArrowDown" => translation[1] += 0.1,
                "ArrowLeft" => translation[0] += 0.1,
                "ArrowRight" => translation[0] -= 0.1,
                _ => {}
            };
        }
        self.draw();
    }

    fn draw(&self) {
        self.context.use_program(Some(&self.program));

        self.context.uniform2fv_with_f32_array(
            Some(&self.translation_uniform_location),
            &mut *self.translation.borrow_mut(),
        );

        self.context
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.buffer));
        self.context.vertex_attrib_pointer_with_i32(
            0,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self.context.enable_vertex_attrib_array(0);

        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        self.context.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            // (vertices.len() / 3) as i32,
            (9 / 3) as i32,
        );
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let app = Rc::new(RefCell::new(App::new("canvas")));

    {
        let for_closure = app.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            for_closure.borrow().on_mouse_down(event);
        }) as Box<dyn FnMut(_)>);
        app.borrow()
            .canvas
            .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let for_closure = app.clone();
        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            for_closure.borrow().on_key_down(event);
        }) as Box<dyn FnMut(_)>);
        let document = web_sys::window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        body.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    app.borrow().draw();

    Ok(())
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error creating shader".into()))
    }
}

pub fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
    context: &WebGlRenderingContext,
    shaders: T,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    for shader in shaders {
        context.attach_shader(&program, shader)
    }
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error creating program object".into()))
    }
}

use wasm_bindgen::prelude::*;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};
use nalgebra::{Matrix4, Rotation3, Vector3};
use std::f32::consts::PI;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[allow(unused_macros)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

// Vertex shader program
const VERTEX_SHADER: &str = r#"
    attribute vec4 aVertexPosition;
    attribute vec4 aVertexColor;
    
    uniform mat4 uModelViewMatrix;
    uniform mat4 uProjectionMatrix;
    
    varying lowp vec4 vColor;
    
    void main() {
        gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
        vColor = aVertexColor;
    }
"#;

// Fragment shader program
const FRAGMENT_SHADER: &str = r#"
    varying lowp vec4 vColor;
    
    void main() {
        gl_FragColor = vColor;
    }
"#;

#[wasm_bindgen]
pub struct Cube {
    gl: WebGlRenderingContext,
    program_info: ProgramInfo,
    buffers: Buffers,
    rotation: f32,
    last_time: f64,
    animation_id: Option<i32>,
    animation_closure: Option<Closure<dyn FnMut(f64)>>,
}

struct ProgramInfo {
    program: WebGlProgram,
    attrib_locations: AttribLocations,
    uniform_locations: UniformLocations,
}

struct AttribLocations {
    vertex_position: u32,
    vertex_color: u32,
}

struct UniformLocations {
    projection_matrix: Option<web_sys::WebGlUniformLocation>,
    model_view_matrix: Option<web_sys::WebGlUniformLocation>,
}

struct Buffers {
    position: web_sys::WebGlBuffer,
    color: web_sys::WebGlBuffer,
    indices: web_sys::WebGlBuffer,
}

#[wasm_bindgen]
impl Cube {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Cube, JsValue> {
        console_error_panic_hook::set_once();
        
        web_sys::console::log_1(&"Cube::new() called".into());
        
        // Get WebGL context
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = match document.get_element_by_id(canvas_id) {
            Some(element) => element,
            None => {
                web_sys::console::error_1(&format!("Could not find canvas with id: {}", canvas_id).into());
                return Err(JsValue::from_str(&format!("Could not find canvas with id: {}", canvas_id)));
            }
        };
        
        web_sys::console::log_1(&"Canvas element found".into());
        
        let canvas: web_sys::HtmlCanvasElement = match canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
            Ok(canvas) => canvas,
            Err(_) => {
                web_sys::console::error_1(&"Element is not a canvas".into());
                return Err(JsValue::from_str("Element is not a canvas"));
            }
        };
        
        let gl = match canvas.get_context("webgl") {
            Ok(Some(ctx)) => match ctx.dyn_into::<WebGlRenderingContext>() {
                Ok(gl) => {
                    web_sys::console::log_1(&"WebGL context created successfully".into());
                    gl
                },
                Err(_) => {
                    web_sys::console::error_1(&"Failed to convert to WebGlRenderingContext".into());
                    return Err(JsValue::from_str("Failed to convert to WebGlRenderingContext"));
                }
            },
            _ => {
                web_sys::console::error_1(&"Failed to get WebGL context".into());
                return Err(JsValue::from_str("Failed to get WebGL context"));
            }
        };
            
        // Initialize shaders and program
        let vert_shader = compile_shader(
            &gl,
            WebGlRenderingContext::VERTEX_SHADER,
            VERTEX_SHADER,
        )?;
        let frag_shader = compile_shader(
            &gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            FRAGMENT_SHADER,
        )?;
        
        let program = link_program(&gl, &vert_shader, &frag_shader)?;
        
        let program_info = ProgramInfo {
            program: program.clone(), // Clone the program to avoid moved value error
            attrib_locations: AttribLocations {
                vertex_position: gl.get_attrib_location(&program, "aVertexPosition") as u32,
                vertex_color: gl.get_attrib_location(&program, "aVertexColor") as u32,
            },
            uniform_locations: UniformLocations {
                projection_matrix: gl.get_uniform_location(&program, "uProjectionMatrix"),
                model_view_matrix: gl.get_uniform_location(&program, "uModelViewMatrix"),
            },
        };
        
        // Create buffers
        let buffers = init_buffers(&gl)?;
        
        // Set clear color and enable depth test
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear_depth(1.0);
        gl.enable(WebGlRenderingContext::DEPTH_TEST);
        gl.depth_func(WebGlRenderingContext::LEQUAL);
        
        let window = web_sys::window().unwrap();
        let performance = window.performance().unwrap();
        
        Ok(Cube {
            gl,
            program_info,
            buffers,
            rotation: 0.0,
            last_time: performance.now(),
            animation_id: None,
            animation_closure: None,
        })
    }
    
    pub fn start(&mut self) -> Result<(), JsValue> {
        // If already running, do nothing
        if self.animation_id.is_some() {
            web_sys::console::log_1(&"Animation already running, ignoring start request".into());
            return Ok(());
        }
        
        web_sys::console::log_1(&"Starting animation...".into());
        
        let cube_ptr = self as *mut Cube;
        
        // Create a new animation closure
        let animation_closure = Closure::wrap(Box::new(move |time: f64| {
            let cube = unsafe { &mut *cube_ptr };
            cube.render(time);
            
            // Request next frame
            let window = web_sys::window().unwrap();
            if let Some(closure) = &cube.animation_closure {
                let id = window.request_animation_frame(closure.as_ref().unchecked_ref()).unwrap();
                cube.animation_id = Some(id);
            } else {
                web_sys::console::warn_1(&"Animation closure is None in the render loop".into());
            }
        }) as Box<dyn FnMut(f64)>);
        
        // Start the animation
        web_sys::console::log_1(&"Requesting first animation frame".into());
        let window = web_sys::window().unwrap();
        match window.request_animation_frame(animation_closure.as_ref().unchecked_ref()) {
            Ok(id) => {
                web_sys::console::log_1(&format!("Animation started with ID: {}", id).into());
                self.animation_id = Some(id);
                self.animation_closure = Some(animation_closure);
                Ok(())
            },
            Err(err) => {
                web_sys::console::error_1(&"Failed to request animation frame".into());
                Err(err)
            }
        }
    }
    
    pub fn stop(&mut self) {
        if let Some(id) = self.animation_id {
            let window = web_sys::window().unwrap();
            window.cancel_animation_frame(id).unwrap();
            self.animation_id = None;
            self.animation_closure = None;
        }
    }
    
    pub fn render(&mut self, time: f64) {
        let delta = time - self.last_time;
        self.last_time = time;
        
        // Update rotation
        self.rotation += (delta as f32) * 0.001;
        
        // Clear the canvas
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);
        
        // Create projection matrix
        let aspect = 1.0;
        let field_of_view = 45.0 * PI / 180.0;
        let z_near = 0.1;
        let z_far = 100.0;
        
        let projection_matrix = Matrix4::new_perspective(aspect, field_of_view, z_near, z_far);
        
        // Create model view matrix
        let mut model_view_matrix = Matrix4::identity();
        
        // Translate the cube
        model_view_matrix = model_view_matrix * Matrix4::new_translation(&Vector3::new(0.0, 0.0, -6.0));
        
        // Rotate the cube
        let rotation = Rotation3::from_euler_angles(self.rotation, self.rotation, self.rotation);
        model_view_matrix = model_view_matrix * rotation.to_homogeneous();
        
        // Draw the cube
        {
            // Positions
            let num_components = 3;
            let normalized = false;
            let stride = 0;
            let offset = 0;
            
            self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.buffers.position));
            self.gl.vertex_attrib_pointer_with_i32(
                self.program_info.attrib_locations.vertex_position,
                num_components,
                WebGlRenderingContext::FLOAT,
                normalized,
                stride,
                offset,
            );
            self.gl.enable_vertex_attrib_array(self.program_info.attrib_locations.vertex_position);
        }
        
        // Colors
        {
            let num_components = 4;
            let normalized = false;
            let stride = 0;
            let offset = 0;
            
            self.gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.buffers.color));
            self.gl.vertex_attrib_pointer_with_i32(
                self.program_info.attrib_locations.vertex_color,
                num_components,
                WebGlRenderingContext::FLOAT,
                normalized,
                stride,
                offset,
            );
            self.gl.enable_vertex_attrib_array(self.program_info.attrib_locations.vertex_color);
        }
        
        // Indices
        self.gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.buffers.indices));
        
        // Use the shader program
        self.gl.use_program(Some(&self.program_info.program));
        
        // Set uniforms
        self.gl.uniform_matrix4fv_with_f32_array(
            self.program_info.uniform_locations.projection_matrix.as_ref(),
            false,
            projection_matrix.as_slice(),
        );
        
        self.gl.uniform_matrix4fv_with_f32_array(
            self.program_info.uniform_locations.model_view_matrix.as_ref(),
            false,
            model_view_matrix.as_slice(),
        );
        
        // Draw elements
        let vertex_count = 36;
        let type_ = WebGlRenderingContext::UNSIGNED_SHORT;
        let offset = 0;
        self.gl.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            vertex_count,
            type_,
            offset,
        );
    }
}

// Helper function to compile a shader
fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    
    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

// Helper function to link a shader program
fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader program"))?;
    
    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);
    
    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program")))
    }
}

// Helper function to initialize buffers
fn init_buffers(gl: &WebGlRenderingContext) -> Result<Buffers, JsValue> {
    // Define vertices for a cube
    let positions = [
        // Front face
        -1.0, -1.0,  1.0,
         1.0, -1.0,  1.0,
         1.0,  1.0,  1.0,
        -1.0,  1.0,  1.0,
        
        // Back face
        -1.0, -1.0, -1.0,
        -1.0,  1.0, -1.0,
         1.0,  1.0, -1.0,
         1.0, -1.0, -1.0,
        
        // Top face
        -1.0,  1.0, -1.0,
        -1.0,  1.0,  1.0,
         1.0,  1.0,  1.0,
         1.0,  1.0, -1.0,
        
        // Bottom face
        -1.0, -1.0, -1.0,
         1.0, -1.0, -1.0,
         1.0, -1.0,  1.0,
        -1.0, -1.0,  1.0,
        
        // Right face
         1.0, -1.0, -1.0,
         1.0,  1.0, -1.0,
         1.0,  1.0,  1.0,
         1.0, -1.0,  1.0,
        
        // Left face
        -1.0, -1.0, -1.0,
        -1.0, -1.0,  1.0,
        -1.0,  1.0,  1.0,
        -1.0,  1.0, -1.0,
    ];
    
    // Define colors for each face
    let colors = [
        // Front face: white
        1.0, 1.0, 1.0, 1.0,
        1.0, 1.0, 1.0, 1.0,
        1.0, 1.0, 1.0, 1.0,
        1.0, 1.0, 1.0, 1.0,
        
        // Back face: red
        1.0, 0.0, 0.0, 1.0,
        1.0, 0.0, 0.0, 1.0,
        1.0, 0.0, 0.0, 1.0,
        1.0, 0.0, 0.0, 1.0,
        
        // Top face: green
        0.0, 1.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 1.0,
        0.0, 1.0, 0.0, 1.0,
        
        // Bottom face: blue
        0.0, 0.0, 1.0, 1.0,
        0.0, 0.0, 1.0, 1.0,
        0.0, 0.0, 1.0, 1.0,
        0.0, 0.0, 1.0, 1.0,
        
        // Right face: yellow
        1.0, 1.0, 0.0, 1.0,
        1.0, 1.0, 0.0, 1.0,
        1.0, 1.0, 0.0, 1.0,
        1.0, 1.0, 0.0, 1.0,
        
        // Left face: purple
        1.0, 0.0, 1.0, 1.0,
        1.0, 0.0, 1.0, 1.0,
        1.0, 0.0, 1.0, 1.0,
        1.0, 0.0, 1.0, 1.0,
    ];
    
    // Define indices to draw the cube
    let indices = [
        0,  1,  2,    0,  2,  3,  // front
        4,  5,  6,    4,  6,  7,  // back
        8,  9,  10,   8,  10, 11, // top
        12, 13, 14,   12, 14, 15, // bottom
        16, 17, 18,   16, 18, 19, // right
        20, 21, 22,   20, 22, 23, // left
    ];
    
    // Create and bind position buffer
    let position_buffer = gl.create_buffer().ok_or("Failed to create position buffer")?;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));
    
    // Pass the vertex positions to WebGL
    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&positions);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
    
    // Create and bind color buffer
    let color_buffer = gl.create_buffer().ok_or("Failed to create color buffer")?;
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&color_buffer));
    
    // Pass the colors to WebGL
    unsafe {
        let colors_array_buf_view = js_sys::Float32Array::view(&colors);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &colors_array_buf_view,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
    
    // Create and bind index buffer
    let index_buffer = gl.create_buffer().ok_or("Failed to create index buffer")?;
    gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    
    // Pass the indices to WebGL
    unsafe {
        let indices_array_buf_view = js_sys::Uint16Array::view(&indices);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &indices_array_buf_view,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
    
    Ok(Buffers {
        position: position_buffer,
        color: color_buffer,
        indices: index_buffer,
    })
} 
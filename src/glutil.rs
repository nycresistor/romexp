use gl;
use gl::types::*;

use std;
use std::str;

pub fn attrib_loc(program : GLuint , name : &str) -> GLint {
    let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();
    let loc = unsafe { gl::GetAttribLocation(program, c_str.as_ptr()) };
    loc
}

pub fn build_shader(src : &str, shader_type : GLenum) -> Option<GLuint> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let src_cstr = std::ffi::CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &src_cstr.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        let mut compiled : GLint = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiled);
        if compiled == gl::TRUE as GLint {
            Some(shader)
        } else {
            gl::DeleteShader(shader);
            None
        }
    }
}

pub fn build_program(vertex_shader_src : &str, fragment_shader_src : &str) -> Option<GLuint> {
    unsafe {
        let program = gl::CreateProgram();
        match (build_shader(vertex_shader_src, gl::VERTEX_SHADER),
               build_shader(fragment_shader_src, gl::FRAGMENT_SHADER)) {
            (Some(vs), Some(fs)) => {
                gl::AttachShader(program, vs);
                gl::AttachShader(program, fs);
                gl::LinkProgram(program);
                gl::DeleteShader(vs);
                gl::DeleteShader(fs);
                let mut linked : GLint = 0;
                gl::GetProgramiv(program, gl::LINK_STATUS, &mut linked);
                if linked == gl::TRUE as GLint {
                    Some(program)
                } else {
                    gl::DeleteProgram(program);
                    None
                }
            },
            _ => {
                gl::DeleteProgram(program);
                None
            }
        }
    }
}

pub fn uniloc(program : GLuint, name : &str) -> GLint {
    let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();
    let loc = unsafe { gl::GetUniformLocation(program, c_str.as_ptr()) };
    loc
}

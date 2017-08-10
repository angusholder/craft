#![feature(conservative_impl_trait)]
#![allow(unused)]

//extern crate byteorder;
extern crate cgmath;
extern crate env_logger;
extern crate fnv;
extern crate image;
#[macro_use]
extern crate imgui;
extern crate imgui_glium_renderer;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate lazy_static;
//extern crate luajit_sys;
#[macro_use]
extern crate log;
//extern crate noise;
extern crate rusqlite;

extern crate deflate;
extern crate inflate;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;


mod block;
mod chunk;
mod chunk_generator;
mod chunk_loader;
mod chunk_manager;
mod craft;
mod math;
mod chunk_mesher;
mod player;
mod utils;


fn main() {
    craft::Craft::run();
}

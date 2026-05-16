use std::path::PathBuf;

use crate::engine::RunningApp;

pub  trait Application{
    fn init(&mut self);
    fn update(&mut self, runtime: &mut RunningApp);
    fn render(&mut self, runtime: &mut RunningApp);
    fn on_resize(&mut self, width: u32, height: u32);
    fn on_drop(&mut self, path: PathBuf);
    fn on_close(&mut self);
    fn exit_requested(&self)->bool;
}
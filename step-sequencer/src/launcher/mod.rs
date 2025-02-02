pub mod launcher_impl;
pub mod launcher_impl2;
pub mod util;

pub use launcher_impl::SSLauncherImpl;
pub use launcher_impl2::SSLauncherImpl2;

use crate::{beatmaker::BeatMakerSubscription, command::Command, project::Project, SSResult};

pub trait SSLauncher: Send {
    fn start(&mut self) -> SSResult<()>;
    fn pause(&mut self) -> SSResult<()>;
    fn stop(&mut self) -> SSResult<()>;
    fn project(&self) -> &Project;
    fn subscribe_to_beats(&self) -> BeatMakerSubscription;
    fn send_command(&self, command: Command) -> SSResult<()>;
}

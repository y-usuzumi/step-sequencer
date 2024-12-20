use step_sequencer::{audio::get_ss_client, SSResult};

fn main() -> SSResult<()> {
    let ss_client = get_ss_client()?;
    ss_client.start()?;
    Ok(())
}

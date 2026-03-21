use super::InputDriver;

pub struct LinuxDriver;

impl InputDriver for LinuxDriver {
    fn press_key(&mut self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
    fn release_key(&mut self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn create() -> anyhow::Result<Self> {
        Ok(LinuxDriver)
    }
}

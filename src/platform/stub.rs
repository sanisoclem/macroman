use super::InputDriver;

pub struct StubDriver;

impl InputDriver for StubDriver {
    fn press_key(&mut self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
    fn release_key(&mut self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn create() -> anyhow::Result<Self> {
        Ok(StubDriver)
    }
}

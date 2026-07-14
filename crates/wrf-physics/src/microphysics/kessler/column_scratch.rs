/// Reusable terminal-velocity storage for one vertical column.
#[derive(Debug)]
pub(super) struct KesslerColumnScratch {
    terminal_velocity: Vec<f32>,
}

impl KesslerColumnScratch {
    pub(super) fn new(level_count: usize) -> Self {
        Self {
            terminal_velocity: vec![0.0; level_count],
        }
    }

    pub(super) fn terminal_velocity_mut(&mut self) -> &mut [f32] {
        &mut self.terminal_velocity
    }
}

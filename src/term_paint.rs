use anyhow::*;

pub struct Painter {
    stdout: Box<term::StdoutTerminal>,
    /// The number of lines output in the previous iteration
    n: usize,
}
impl Painter {
    pub fn new() -> Result<Painter> {
        Ok(Painter {
            stdout: term::stdout().ok_or_else(|| anyhow!("Couldn't open stdout as a terminal"))?,
            n: 0,
        })
    }

    /// Clear the previous output and replace it with the new output
    pub fn print(&mut self, out: &[u8]) -> Result<()> {
        for _ in 0..self.n {
            self.stdout.cursor_up()?;
            self.stdout.delete_line()?;
        }
        self.stdout.write_all(&out)?;
        self.n = out.into_iter().filter(|c| **c == b'\n').count();
        Ok(())
    }
}

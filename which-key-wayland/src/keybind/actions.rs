use std::process;
use std::process::Stdio;

use kdl::KdlNode;

const ACTION_NAMES: &[&str] = &["spawn", "sh"];

#[derive(Debug, Clone)]
pub enum Action {
    Spawn(Spawn),
    Sh(Sh),
}

impl Action {
    pub fn is_action(node: &KdlNode) -> bool {
        ACTION_NAMES.contains(&node.name().value())
    }

    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Action::Spawn(s) => s.run(),
            Action::Sh(s) => s.run(),
        }
    }

    pub fn parse(node: &KdlNode) -> anyhow::Result<Self> {
        let args: Vec<_> = node
            .entries()
            .iter()
            .filter_map(|e| e.value().as_string().map(String::from))
            .collect();

        match node.name().value() {
            "spawn" => {
                let mut args = args.into_iter();
                let Some(program) = args.next() else {
                    anyhow::bail!("`spawn` requires at least 1 argument");
                };
                let args = args.collect();
                Ok(Self::Spawn(Spawn::new(program, args)))
            }
            "sh" => {
                let shell = String::from("sh");
                let command = args.into_iter().collect();
                Ok(Self::Sh(Sh::new(shell, command)))
            }
            invalid => anyhow::bail!("Invalid action: {invalid}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spawn {
    program: String,
    args: Vec<String>,
}

impl Spawn {
    pub fn new(program: String, args: Vec<String>) -> Self {
        Self { program, args }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        process::Command::new(&self.program)
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Sh {
    shell: String,
    command: String,
}

impl Sh {
    pub fn new(shell: String, command: String) -> Self {
        Self { shell, command }
    }

    pub fn run(&self) -> anyhow::Result<()> {
        process::Command::new(&self.shell)
            .arg("-c")
            .arg(&self.command)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }
}

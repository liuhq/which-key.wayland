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

    pub fn shell(&self) -> &str {
        &self.shell
    }

    pub fn command(&self) -> &str {
        &self.command
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

impl Spawn {
    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kdl::{KdlDocument, KdlNode};

    fn parse_kdl_nodes(source: &str) -> Vec<KdlNode> {
        let doc: KdlDocument = source.parse().unwrap();
        doc.nodes().to_vec()
    }

    fn first_node(source: &str) -> KdlNode {
        parse_kdl_nodes(source).into_iter().next().unwrap()
    }

    #[test]
    fn is_action_true_for_spawn() {
        let node = first_node("spawn \"firefox\"");
        assert!(Action::is_action(&node));
    }

    #[test]
    fn is_action_true_for_sh() {
        let node = first_node("sh \"echo hello\"");
        assert!(Action::is_action(&node));
    }

    #[test]
    fn is_action_false_for_unknown() {
        let node = first_node("unknown \"foo\"");
        assert!(!Action::is_action(&node));
    }

    #[test]
    fn is_action_false_for_empty_name() {
        let node = first_node("exec \"foo\"");
        assert!(!Action::is_action(&node));
    }

    #[test]
    fn parse_spawn_single_arg() {
        let node = first_node("spawn \"firefox\"");
        let action = Action::parse(&node).unwrap();
        match action {
            Action::Spawn(s) => {
                assert_eq!(s.program(), "firefox");
                assert!(s.args().is_empty());
            }
            _ => panic!("Expected Spawn action"),
        }
    }

    #[test]
    fn parse_spawn_multiple_args() {
        let node = first_node("spawn \"firefox\" \"--new-window\" \"https://example.com\"");
        let action = Action::parse(&node).unwrap();
        match action {
            Action::Spawn(s) => {
                assert_eq!(s.program(), "firefox");
                assert_eq!(s.args(), &["--new-window", "https://example.com"]);
            }
            _ => panic!("Expected Spawn action"),
        }
    }

    #[test]
    fn parse_spawn_no_args_error() {
        let node = first_node("spawn");
        let result = Action::parse(&node);
        assert!(result.is_err());
    }

    #[test]
    fn parse_sh_single_arg() {
        let node = first_node("sh \"echo hello\"");
        let action = Action::parse(&node).unwrap();
        match action {
            Action::Sh(s) => {
                assert_eq!(s.shell(), "sh");
                assert_eq!(s.command(), "echo hello");
            }
            _ => panic!("Expected Sh action"),
        }
    }

    #[test]
    fn parse_sh_multiple_args_joined() {
        let node = first_node("sh \"echo\" \"hello\" \"world\"");
        let action = Action::parse(&node).unwrap();
        match action {
            Action::Sh(s) => {
                assert_eq!(s.shell(), "sh");
                assert_eq!(s.command(), "echohelloworld");
            }
            _ => panic!("Expected Sh action"),
        }
    }

    #[test]
    fn parse_sh_no_args() {
        let node = first_node("sh");
        let action = Action::parse(&node).unwrap();
        match action {
            Action::Sh(s) => {
                assert_eq!(s.shell(), "sh");
                assert_eq!(s.command(), "");
            }
            _ => panic!("Expected Sh action"),
        }
    }

    #[test]
    fn parse_unknown_action_name_error() {
        let node = first_node("unknown \"arg\"");
        let result = Action::parse(&node);
        assert!(result.is_err());
    }

    #[test]
    fn spawn_new_constructor() {
        let s = Spawn::new("firefox".into(), vec!["--new-window".into()]);
        assert_eq!(s.program(), "firefox");
        assert_eq!(s.args(), &["--new-window"]);
    }

    #[test]
    fn sh_new_constructor() {
        let s = Sh::new("bash".into(), "echo hello".into());
        assert_eq!(s.shell(), "bash");
        assert_eq!(s.command(), "echo hello");
    }
}

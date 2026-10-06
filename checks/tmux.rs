use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

#[allow(dead_code)]
#[path = "support.rs"]
mod support;

pub struct Terminal {
    pub root: support::TempDir,
    pub binary: PathBuf,
    pub config: PathBuf,
    pub session: String,
    pub environment: BTreeMap<OsString, OsString>,
    socket: String,
}

pub fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

pub fn sleep(seconds: f64) {
    thread::sleep(Duration::from_secs_f64(seconds));
}

impl Terminal {
    pub fn new(package: &std::path::Path, session: &str) -> Self {
        let root = support::TempDir::new();
        let mut environment = env::vars_os()
            .filter(|(key, _)| {
                !["ZELLIJ", "YZX_", "YAZELIX_"]
                    .iter()
                    .any(|prefix| key.to_string_lossy().starts_with(prefix))
            })
            .collect::<BTreeMap<_, _>>();
        for (key, value) in [
            ("HOME", root.path.join("home")),
            ("XDG_CACHE_HOME", root.path.join("cache")),
            ("XDG_DATA_HOME", root.path.join("data")),
            ("ZELLIJ_SOCKET_DIR", root.path.join("s")),
        ] {
            std::fs::create_dir_all(&value).unwrap();
            environment.insert(key.into(), value.into_os_string());
        }
        environment.insert("TERM".into(), "xterm-256color".into());
        let socket = root
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        Self {
            root,
            binary: package.join("bin/yzx-zellij"),
            config: package.join("share/yazelix/config.kdl"),
            session: session.into(),
            environment,
            socket,
        }
    }

    pub fn command(&self, program: impl AsRef<std::ffi::OsStr>) -> Command {
        let mut command = Command::new("timeout");
        command
            .args(["--kill-after=1s", "20s"])
            .arg(program)
            .env_clear()
            .envs(&self.environment);
        command
    }

    pub fn run(
        &self,
        program: impl AsRef<std::ffi::OsStr>,
        args: &[&str],
        input: Option<&str>,
    ) -> String {
        let mut command = self.command(program.as_ref());
        command
            .args(args)
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().unwrap();
        if let Some(input) = input {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(input.as_bytes())
                .unwrap();
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{:?} {args:?}: {}\n{}",
            program.as_ref(),
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    pub fn tmux(&self, args: &[&str]) -> String {
        let mut all = vec!["-f", "/dev/null", "-L", &self.socket];
        all.extend_from_slice(args);
        self.run("tmux", &all, None)
    }

    pub fn capture(&self, target: &str, ansi: bool) -> String {
        let mut args = vec!["capture-pane", "-t", target, "-p"];
        if ansi {
            args.push("-e");
        }
        self.tmux(&args)
    }

    pub fn resize(&self, target: &str, width: usize, height: usize) {
        self.tmux(&[
            "resize-window",
            "-t",
            target,
            "-x",
            &width.to_string(),
            "-y",
            &height.to_string(),
        ]);
    }

    pub fn action_command(&self, args: &[&str]) -> Command {
        let mut command = self.command(&self.binary);
        command
            .arg("-c")
            .arg(&self.config)
            .args(["-s", &self.session, "action"])
            .args(args);
        command
    }

    pub fn action(&self, args: &[&str]) -> String {
        support::successful_stdout(&mut self.action_command(args), "native action")
    }

    pub fn launch(&self, args: &[&str]) -> String {
        [
            &*self.binary.to_string_lossy(),
            "-c",
            &*self.config.to_string_lossy(),
        ]
        .into_iter()
        .chain(args.iter().copied())
        .map(quote)
        .collect::<Vec<_>>()
        .join(" ")
    }

    pub fn wait(&self, mut check: impl FnMut() -> bool, context: &str, target: &str) {
        for _ in 0..100 {
            if check() {
                return;
            }
            sleep(0.1);
        }
        panic!("{context}\n{}", self.capture(target, true));
    }

    pub fn stop(&self) {
        for session in [&self.session, &format!("{}-rendering", self.session)] {
            let _ = self
                .command(&self.binary)
                .args(["kill-session", session])
                .output();
        }
        let _ = self
            .command("tmux")
            .args(["-f", "/dev/null", "-L", &self.socket, "kill-server"])
            .output();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.stop();
    }
}

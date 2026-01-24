pub const DEFAULT_CONFIG: &str = r#"paths:
  workspace: ~/workspace
  inbox: ~/workspace/0_Inbox
  projects: ~/workspace/1_Projects
  areas: ~/workspace/2_Areas
  resources: ~/workspace/3_Resources
  archives: ~/workspace/4_Archives

organize:
  ctf_dir: projects/CTFs

ctf:
  default_categories:
    - web
    - pwn
    - crypto
    - rev
    - misc
"#;

pub const SOLVE_PY_PWN: &str = r#"from pwn import *

# io = process('./chall')
io = remote('TARGET', PORT)

io.interactive()
"#;

pub const SOLVE_PY_WEB: &str = r#"import requests

URL = "http://TARGET"

r = requests.get(URL)
print(r.text)
"#;

pub const SOLVE_PY_GENERIC: &str = r#"# Solve script for challenge
"#;

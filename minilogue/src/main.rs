//! Korg minilogue CLI: MIDI dump/sync over USB port 2, and `.mnlgprog` interop.

mod midi;

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use minilogue_core::pack::{pack, unpack};
use minilogue_core::yaml::{
    global_from_yaml_str, global_to_yaml_string, program_from_yaml_str, program_to_yaml_string,
    GLOBAL_YAML_HEADER, PROGRAM_YAML_HEADER,
};
use minilogue_core::{Function, GlobalArea, MnlgProgram, ProgInfo, Program};

use crate::midi::MidiSession;

const DUMP_TIMEOUT: Duration = Duration::from_secs(3);
const SYNC_TIMEOUT: Duration = Duration::from_secs(4);
const MAX_PROGRAM: u16 = 199;

#[derive(Parser, Debug)]
#[command(name = "minilogue", version, about = "Korg minilogue tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Debug, Clone)]
struct ConnOpts {
    /// MIDI port name substring (default prefers the SysEx port 2).
    #[arg(long)]
    port: Option<String>,
    /// 1-based MIDI channel (the synth's global channel).
    #[arg(long, default_value_t = 1)]
    channel: u8,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct Target {
    /// The edit buffer (current program).
    #[arg(long)]
    current: bool,
    /// A stored slot, 0..=199.
    #[arg(long, value_name = "N")]
    program: Option<u16>,
    /// The global area.
    #[arg(long)]
    global: bool,
    /// All: the global area plus every stored slot (to/from a directory).
    #[arg(long)]
    all: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// List available MIDI input and output ports.
    Ports,
    /// Probe the device identity (Universal Identity Request).
    Identity {
        #[command(flatten)]
        conn: ConnOpts,
    },
    /// Read program/global data from the synth into YAML.
    Dump {
        #[command(flatten)]
        conn: ConnOpts,
        #[command(flatten)]
        target: Target,
        /// Output YAML file (or directory for `--all`).
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
    /// Write YAML program/global data to the synth.
    Sync {
        #[command(flatten)]
        conn: ConnOpts,
        #[command(flatten)]
        target: Target,
        /// Input YAML file (or directory for `--all`).
        #[arg(short = 'i', long)]
        input: PathBuf,
        /// Read back after writing and confirm it matches.
        #[arg(long)]
        verify: bool,
    },
    /// Validate and summarise a YAML file.
    Show { path: PathBuf },
    /// Print a JSON Schema (`global` or `program`).
    Schema { kind: String },
    /// `.mnlgprog` / `.mnlglib` interop.
    Mnlg {
        #[command(subcommand)]
        cmd: MnlgCmd,
    },
}

#[derive(Subcommand, Debug)]
enum MnlgCmd {
    /// Import a `.mnlgprog`/`.mnlglib` to YAML (file → file, library → directory).
    Import {
        archive: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
    /// Export a program YAML to a `.mnlgprog`.
    Export {
        yaml: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
    /// Bundle a directory of program YAMLs into a `.mnlglib`.
    Lib {
        dir: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    match Cli::parse().command {
        Command::Ports => list_ports(),
        Command::Identity { conn } => identity(&conn),
        Command::Dump {
            conn,
            target,
            output,
        } => dump(&conn, &target, &output),
        Command::Sync {
            conn,
            target,
            input,
            verify,
        } => sync(&conn, &target, &input, verify),
        Command::Show { path } => show(&path),
        Command::Schema { kind } => print_schema(&kind),
        Command::Mnlg { cmd } => mnlg(cmd),
    }
}

fn open(conn: &ConnOpts) -> Result<MidiSession> {
    let s = MidiSession::open(conn.port.as_deref(), conn.channel)?;
    log::info!("in: {}  out: {}", s.in_name, s.out_name);
    Ok(s)
}

fn program_number_bytes(n: u16) -> Vec<u8> {
    vec![(n & 0x7F) as u8, ((n >> 7) & 0x01) as u8]
}

// --- MIDI dump ---

fn dump_current(s: &mut MidiSession) -> Result<Program> {
    let f = s.request(
        Function::CurrentProgramDumpRequest.code(),
        vec![],
        Function::CurrentProgramDump.code(),
        DUMP_TIMEOUT,
    )?;
    Ok(Program::from_bytes(&unpack(&f.data))?)
}

fn dump_program(s: &mut MidiSession, n: u16) -> Result<Program> {
    let f = s.request(
        Function::ProgramDumpRequest.code(),
        program_number_bytes(n),
        Function::ProgramDump.code(),
        DUMP_TIMEOUT,
    )?;
    // 0x4C reply echoes the 2-byte program number before the packed payload.
    Ok(Program::from_bytes(&unpack(&f.data[2..]))?)
}

fn dump_global(s: &mut MidiSession) -> Result<GlobalArea> {
    let f = s.request(
        Function::GlobalDumpRequest.code(),
        vec![],
        Function::GlobalDump.code(),
        DUMP_TIMEOUT,
    )?;
    Ok(GlobalArea::from_bytes(&unpack(&f.data))?)
}

fn dump(conn: &ConnOpts, target: &Target, output: &Path) -> Result<()> {
    let mut s = open(conn)?;
    if target.all {
        std::fs::create_dir_all(output)?;
        let g = dump_global(&mut s)?;
        write_global_yaml(&output.join("global.yaml"), &g)?;
        for n in 0..=MAX_PROGRAM {
            let p = dump_program(&mut s, n).with_context(|| format!("dumping program {n}"))?;
            write_program_yaml(&output.join(format!("program_{n:03}.yaml")), &p)?;
        }
        println!(
            "dumped global + {} programs to {}",
            MAX_PROGRAM + 1,
            output.display()
        );
    } else if target.global {
        let g = dump_global(&mut s)?;
        write_global_yaml(output, &g)?;
        println!("dumped global -> {}", output.display());
    } else if target.current {
        let p = dump_current(&mut s)?;
        write_program_yaml(output, &p)?;
        println!("dumped current ({}) -> {}", p.name, output.display());
    } else if let Some(n) = target.program {
        check_program(n)?;
        let p = dump_program(&mut s, n)?;
        write_program_yaml(output, &p)?;
        println!("dumped program {n} ({}) -> {}", p.name, output.display());
    }
    Ok(())
}

// --- MIDI sync (write) ---

fn sync(conn: &ConnOpts, target: &Target, input: &Path, verify: bool) -> Result<()> {
    let mut s = open(conn)?;
    if target.all {
        let g = read_global_yaml(&input.join("global.yaml"))?;
        s.send_and_ack(
            Function::GlobalDump.code(),
            pack(&g.to_bytes()?),
            SYNC_TIMEOUT,
        )?;
        for n in 0..=MAX_PROGRAM {
            let path = input.join(format!("program_{n:03}.yaml"));
            if !path.exists() {
                continue;
            }
            let p = read_program_yaml(&path)?;
            sync_program(&mut s, n, &p, verify)?;
        }
        println!("synced global + programs from {}", input.display());
    } else if target.global {
        let g = read_global_yaml(input)?;
        s.send_and_ack(
            Function::GlobalDump.code(),
            pack(&g.to_bytes()?),
            SYNC_TIMEOUT,
        )?;
        println!("synced global");
    } else if target.current {
        let p = read_program_yaml(input)?;
        s.send_and_ack(
            Function::CurrentProgramDump.code(),
            pack(&p.to_bytes()?),
            SYNC_TIMEOUT,
        )?;
        if verify {
            let back = dump_current(&mut s)?;
            ensure_match(&p, &back)?;
        }
        println!("synced current ({})", p.name);
    } else if let Some(n) = target.program {
        check_program(n)?;
        let p = read_program_yaml(input)?;
        sync_program(&mut s, n, &p, verify)?;
        println!("synced program {n} ({})", p.name);
    }
    Ok(())
}

fn sync_program(s: &mut MidiSession, n: u16, p: &Program, verify: bool) -> Result<()> {
    let mut data = program_number_bytes(n);
    data.extend_from_slice(&pack(&p.to_bytes()?));
    s.send_and_ack(Function::ProgramDump.code(), data, SYNC_TIMEOUT)?;
    if verify {
        let back = dump_program(s, n)?;
        ensure_match(p, &back).with_context(|| format!("verifying program {n}"))?;
    }
    Ok(())
}

fn ensure_match(want: &Program, got: &Program) -> Result<()> {
    if want.to_bytes()? != got.to_bytes()? {
        bail!("verify failed: read-back differs from what was written");
    }
    Ok(())
}

// --- identity ---

fn identity(conn: &ConnOpts) -> Result<()> {
    let mut s = open(conn)?;
    let r = s.identity(Duration::from_secs(2))?;
    let hex = r
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ");
    println!("identity reply: {hex}");
    // F0 7E cc 06 02 <manuf> <family LSB MSB> <member LSB MSB> <version..> F7
    if r.len() >= 8 && r[5] == 0x42 {
        println!("  manufacturer: Korg (0x42)");
        println!(
            "  family: {:02X} {:02X}  member: {:02X} {:02X}",
            r[6], r[7], r[8], r[9]
        );
        if r.len() >= 14 {
            println!(
                "  version bytes: {:02X} {:02X} {:02X} {:02X}",
                r[10], r[11], r[12], r[13]
            );
        }
    }
    Ok(())
}

// --- file helpers ---

fn check_program(n: u16) -> Result<()> {
    if n > MAX_PROGRAM {
        bail!("program number {n} out of range (0..={MAX_PROGRAM})");
    }
    Ok(())
}

fn write_yaml(path: &Path, header: &str, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, format!("{header}\n{body}"))
        .with_context(|| format!("writing {}", path.display()))
}

fn write_program_yaml(path: &Path, p: &Program) -> Result<()> {
    write_yaml(path, PROGRAM_YAML_HEADER, &program_to_yaml_string(p)?)
}

fn write_global_yaml(path: &Path, g: &GlobalArea) -> Result<()> {
    write_yaml(path, GLOBAL_YAML_HEADER, &global_to_yaml_string(g)?)
}

fn read_program_yaml(path: &Path) -> Result<Program> {
    let s = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    program_from_yaml_str(&s).with_context(|| format!("parsing program {}", path.display()))
}

fn read_global_yaml(path: &Path) -> Result<GlobalArea> {
    let s = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    global_from_yaml_str(&s).with_context(|| format!("parsing global {}", path.display()))
}

fn show(path: &Path) -> Result<()> {
    let s = std::fs::read_to_string(path)?;
    if let Ok(p) = program_from_yaml_str(&s) {
        println!("program: {}  voice_mode: {:?}", p.name, p.voice_mode);
    } else if let Ok(g) = global_from_yaml_str(&s) {
        println!(
            "global: midi_channel {}  brightness {}",
            g.midi_channel, g.brightness
        );
    } else {
        bail!("not a valid program or global YAML: {}", path.display());
    }
    Ok(())
}

fn print_schema(kind: &str) -> Result<()> {
    let schema = match kind {
        "global" => minilogue_core::schema::global_area_schema(),
        "program" => minilogue_core::schema::program_schema(),
        other => bail!("unknown schema {other:?} (known: global, program)"),
    };
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}

// --- mnlg interop ---

fn mnlg(cmd: MnlgCmd) -> Result<()> {
    match cmd {
        MnlgCmd::Import { archive, output } => {
            let bytes = std::fs::read(&archive)?;
            let progs = minilogue_core::mnlg::read_library(&bytes)
                .with_context(|| format!("reading {}", archive.display()))?;
            if progs.len() == 1 {
                write_program_yaml(&output, &progs[0].program)?;
                println!("imported {} -> {}", progs[0].program.name, output.display());
            } else {
                std::fs::create_dir_all(&output)?;
                for (i, mp) in progs.iter().enumerate() {
                    write_program_yaml(&output.join(format!("program_{i:03}.yaml")), &mp.program)?;
                }
                println!("imported {} programs -> {}", progs.len(), output.display());
            }
        }
        MnlgCmd::Export { yaml, output } => {
            let p = read_program_yaml(&yaml)?;
            let bytes = minilogue_core::write_mnlgprog(&p, &ProgInfo::default())?;
            std::fs::write(&output, bytes)?;
            println!("exported {} -> {}", p.name, output.display());
        }
        MnlgCmd::Lib { dir, output } => {
            let mut yamls: Vec<PathBuf> = std::fs::read_dir(&dir)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "yaml" || x == "yml"))
                .collect();
            yamls.sort();
            if yamls.is_empty() {
                bail!("no .yaml files in {}", dir.display());
            }
            let progs: Result<Vec<MnlgProgram>> = yamls
                .iter()
                .map(|p| {
                    Ok(MnlgProgram {
                        program: read_program_yaml(p)?,
                        info: ProgInfo::default(),
                    })
                })
                .collect();
            let bytes = minilogue_core::mnlg::write_library(&progs?)?;
            std::fs::write(&output, bytes)?;
            println!("bundled {} programs -> {}", yamls.len(), output.display());
        }
    }
    Ok(())
}

fn list_ports() -> Result<()> {
    use midir::{MidiInput, MidiOutput};
    let input = MidiInput::new("minilogue-ports-in")?;
    println!("Input ports:");
    for (i, port) in input.ports().iter().enumerate() {
        println!(
            "  [{i}] {}",
            input.port_name(port).unwrap_or_else(|_| "<unknown>".into())
        );
    }
    let output = MidiOutput::new("minilogue-ports-out")?;
    println!("Output ports:");
    for (i, port) in output.ports().iter().enumerate() {
        println!(
            "  [{i}] {}",
            output
                .port_name(port)
                .unwrap_or_else(|_| "<unknown>".into())
        );
    }
    Ok(())
}

//! MIDI session over `midir`, targeting the minilogue's USB **port 2**
//! (`MIDIOUT2`/`MIDIIN2`) where bulk SysEx lives. Port 1 ignores dumps.

use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use midir::{
    MidiInput, MidiInputConnection, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort,
};
use minilogue_core::Frame;

/// ACK / NAK status function codes.
const ACK: u8 = 0x23;
const NAK: u8 = 0x24;
const FORMAT_ERR: u8 = 0x26;

pub struct MidiSession {
    out: MidiOutputConnection,
    rx: Receiver<Vec<u8>>,
    _conn_in: MidiInputConnection<()>,
    channel_byte: u8,
    pub in_name: String,
    pub out_name: String,
}

fn pick_input(mi: &MidiInput, needles: &[&str]) -> Option<MidiInputPort> {
    for needle in needles {
        for p in mi.ports() {
            if matches!(mi.port_name(&p), Ok(n) if n.to_lowercase().contains(&needle.to_lowercase()))
            {
                return Some(p);
            }
        }
    }
    None
}

fn pick_output(mo: &MidiOutput, needles: &[&str]) -> Option<MidiOutputPort> {
    for needle in needles {
        for p in mo.ports() {
            if matches!(mo.port_name(&p), Ok(n) if n.to_lowercase().contains(&needle.to_lowercase()))
            {
                return Some(p);
            }
        }
    }
    None
}

impl MidiSession {
    /// Open the SysEx-capable port pair. `port_hint` overrides the default
    /// (which prefers the port-2 `MIDIIN2`/`MIDIOUT2` interface).
    pub fn open(port_hint: Option<&str>, channel: u8) -> Result<Self> {
        if !(1..=16).contains(&channel) {
            bail!("channel must be 1..=16");
        }
        let mi = MidiInput::new("minilogue-cli-in")?;
        let mo = MidiOutput::new("minilogue-cli-out")?;

        let in_needles: Vec<&str> = match port_hint {
            Some(h) => vec![h],
            None => vec!["MIDIIN2", "minilogue"],
        };
        let out_needles: Vec<&str> = match port_hint {
            Some(h) => vec![h],
            None => vec!["MIDIOUT2", "minilogue"],
        };
        let in_port = pick_input(&mi, &in_needles).ok_or_else(|| {
            anyhow!("no MIDI input matches {in_needles:?} (try --port, or `ports`)")
        })?;
        let out_port = pick_output(&mo, &out_needles)
            .ok_or_else(|| anyhow!("no MIDI output matches {out_needles:?} (try --port)"))?;
        let in_name = mi.port_name(&in_port)?;
        let out_name = mo.port_name(&out_port)?;

        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let conn_in = mi
            .connect(
                &in_port,
                "minilogue-in",
                move |_, msg, _| {
                    let _ = tx.send(msg.to_vec());
                },
                (),
            )
            .map_err(|e| anyhow!("opening input port: {e}"))?;
        let out = mo
            .connect(&out_port, "minilogue-out")
            .map_err(|e| anyhow!("opening output port: {e}"))?;

        Ok(Self {
            out,
            rx,
            _conn_in: conn_in,
            channel_byte: 0x30 | ((channel - 1) & 0x0F),
            in_name,
            out_name,
        })
    }

    fn drain(&self) {
        while self.rx.try_recv().is_ok() {}
    }

    fn send_raw(&mut self, bytes: &[u8]) -> Result<()> {
        self.out.send(bytes).map_err(|e| anyhow!("MIDI send: {e}"))
    }

    /// Send a Korg function frame.
    pub fn send(&mut self, function: u8, data: Vec<u8>) -> Result<()> {
        let frame = Frame::new(self.channel_byte, function, data);
        self.send_raw(&frame.encode())
    }

    fn recv_until<T>(
        &self,
        timeout: Duration,
        mut accept: impl FnMut(&[u8]) -> Option<T>,
    ) -> Result<T> {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                bail!("timed out after {timeout:?} waiting for device");
            }
            match self.rx.recv_timeout(remaining) {
                Ok(raw) => {
                    if let Some(v) = accept(&raw) {
                        return Ok(v);
                    }
                }
                Err(_) => bail!("timed out after {timeout:?} waiting for device"),
            }
        }
    }

    /// Send a request and return the reply frame with `reply_func`.
    pub fn request(
        &mut self,
        function: u8,
        data: Vec<u8>,
        reply_func: u8,
        timeout: Duration,
    ) -> Result<Frame> {
        self.drain();
        self.send(function, data)?;
        self.recv_until(timeout, |raw| {
            Frame::decode(raw).ok().filter(|f| f.function == reply_func)
        })
    }

    /// Send data and wait for the device's ACK (`0x23`), erroring on NAK.
    pub fn send_and_ack(&mut self, function: u8, data: Vec<u8>, timeout: Duration) -> Result<()> {
        self.drain();
        self.send(function, data)?;
        self.recv_until(timeout, |raw| match Frame::decode(raw) {
            Ok(f) if f.function == ACK => Some(Ok(())),
            Ok(f) if f.function == NAK => Some(Err("DATA LOAD ERROR (NAK)")),
            Ok(f) if f.function == FORMAT_ERR => Some(Err("DATA FORMAT ERROR")),
            _ => None,
        })?
        .map_err(|e| anyhow!("device rejected data: {e}"))
    }

    /// Send the Universal Identity Request and return the raw reply bytes.
    pub fn identity(&mut self, timeout: Duration) -> Result<Vec<u8>> {
        self.drain();
        self.send_raw(&[0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7])?;
        self.recv_until(timeout, |raw| {
            (raw.len() >= 6 && raw[0] == 0xF0 && raw[1] == 0x7E && raw[3] == 0x06 && raw[4] == 0x02)
                .then(|| raw.to_vec())
        })
    }
}

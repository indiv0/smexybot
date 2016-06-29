// Copyright (c) 2016 Nikita Pekin and the smexybot contributors
// See the README.md file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate byteorder;
extern crate cpal;
extern crate futures;

use std;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::io::{
    self,
    Cursor,
    ErrorKind as IoErrorKind,
    Read,
    Result as IoResult,
    Write,
};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

use self::byteorder::{
    LittleEndian,
    ReadBytesExt,
    WriteBytesExt,
};
use self::cpal::Voice;
use self::futures::Stream;
use self::futures::task::{self, Executor, Run};
use serde_json;
use serde_json::Value;
use serenity::{Error, Result as SerenityResult};
use serenity::client::{CACHE, Context};
use serenity::ext::voice::{
    self,
    AudioReceiver,
    AudioSource,
    VoiceError,
};
use serenity::model::{ChannelId, Mentionable, Message};

use util::check_msg;

pub fn join(context: &Context, message: &Message, args: Vec<String>)
    -> Result<(), String>
{
    let connect_to = match args.get(0) {
        Some(arg) => match arg.parse::<u64>() {
            Ok(id) => ChannelId(id),
            Err(_why) => {
                check_msg(message.reply("Invalid voice channel ID given"));

                return Ok(());
            },
        },
        None => {
            check_msg(message.reply("Requires a voice channel ID be given"));

            return Ok(());
        },
    };

    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Groups and DMs not supported"));

            return Ok(());
        },
    };

    let mut shard = context.shard.lock().unwrap();
    shard.manager.join(Some(guild_id), connect_to);

    check_msg(context.say(&format!("Joined {}", connect_to.mention())));

    Ok(())
}

pub fn listen(context: &Context, message: &Message, _args: Vec<String>)
    -> Result<(), String>
{
    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Error finding channel info"));

            return Ok(());
        },
    };

    if let Some(handler) = context.shard.lock().unwrap().manager.get(guild_id) {
        handler.listen(Box::new(MyReceiver::new()) as Box<AudioReceiver>);

        check_msg(context.say("Listening"));
    } else {
        check_msg(context.say("Not in a voice channel to listen in"));
    }

    Ok(())
}

pub fn unlisten(context: &Context, message: &Message, _args: Vec<String>)
    -> Result<(), String>
{
    let guild_id = match CACHE.read().unwrap().get_guild_channel(message.channel_id) {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(context.say("Error finding channel info"));

            return Ok(());
        },
    };

    if let Some(handler) = context.shard.lock().unwrap().manager.get(guild_id) {
        handler.listen(None);

        check_msg(context.say("Not listening"));
    } else {
        check_msg(context.say("Not in a voice channel to listen in"));
    }

    Ok(())
}

struct MyReceiver {
    //player: Player,
    data: Vec<i16>,
    command: Option<Child>,
}

impl MyReceiver {
    fn new() -> Self {
        MyReceiver {
            //player: Player::new(),
            data: Vec::new(),
            command: None,
        }
    }
}

impl AudioReceiver for MyReceiver {
    fn speaking_update(&mut self, ssrc: u32, user_id: u64, speaking: bool) {
        debug!("Speaking update: {}, {}, {}", ssrc, user_id, speaking);

        if speaking {
            debug!("Playing");
            //self.player.voice.play();
            self.data = Vec::new();
            self.command = Some(create_ffmpeg_command("./data.wav").unwrap());
        } else {
            debug!("Pausing");
            //self.player.voice.pause();

            //let mut player = Player::new(&mut self.data);
            //player.voice.play();

            /*
            let mut audio = Audio::new();
            debug!("Sample rate: {}", audio.sample_rate());
            audio.add_samples(self.data.iter().cloned().collect());
            */

            //ffmpeg("./data.wav", self.data.as_slice());

            {
                debug!("Closing stdin");
                let mut command = self.command.as_mut().unwrap();
                let mut stdin = command.stdin.take().unwrap();
                drop(stdin);
                debug!("Closed stdin");

                let mut stderr = command.stderr.as_mut().unwrap();
                let mut stdout = command.stdout.as_mut().unwrap();
                let mut stderr_str = String::new();
                let mut stdout_str = String::new();
                stderr.read_to_string(&mut stderr_str).unwrap();
                stdout.read_to_string(&mut stdout_str).unwrap();
                debug!("Read from stderr/stdout");

                debug!("stderr: {}", stderr_str);
                debug!("stdout: {}", stdout_str);
            }
            self.command = None;
        }
    }

    fn voice_packet(&mut self, _ssrc: u32, sequence: u16, _timestamp: u32, _stereo: bool, data: &[i16]) {
        debug!("Voice packet: {}", sequence);
        self.data.extend(data.iter().cloned());

        if self.command.is_some() {
            let mut buffer: &[u8] = unsafe { ::std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2) };
            {
                let mut stdin = self.command.as_mut()
                    .unwrap()
                    .stdin.as_mut()
                    .unwrap();
                /*
                for v in data.iter() {
                    stdin.write_i16::<LittleEndian>(*v);
                }
                */
                io::copy(&mut buffer, &mut stdin).expect("Failed to write data to ffmpeg pipe");
            }
        }
    }
}

struct AudioExecutor;

impl Executor for AudioExecutor {
    fn execute(&self, r: Run) {
        r.run();
    }
}

struct Player {
    voice: Voice,
}

impl Player {
    fn new(data_source: &mut Vec<i16>) -> Self {
        let endpoint = cpal::get_default_endpoint().expect("Failed to get default endpoint");
        let format = endpoint.get_supported_formats_list().unwrap().next().expect("Failed to get endpoint format");
        debug!("Created endpoint & determined format");

        let event_loop = cpal::EventLoop::new();
        let executor = Arc::new(AudioExecutor);
        debug!("Initialized event loop and executor");

        let (voice, stream) = cpal::Voice::new(&endpoint, &format, &event_loop).expect("Failed to create a voice");
        debug!("Created a voice and stream");

        // Produces a sinusoid of maximum amplitude.
        let samples_rate = format.samples_rate.0 as f32;
        /*
        let mut data_source: Vec<u64> = (0u64..).map(move |t| t as f32 * 440.0 * 2.0 * std::f32::consts::PI / samples_rate) // 440 Hz
            .map(move |t| t.sin());
        debug!("Generated sinusoid data stream");
        */

        //voice.play();

        /*
        task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
            match buffer {
                /*
                cpal::UnknownTypeBuffer::U16(mut buffer) => {
                    for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                        let value = ((value * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
                        for out in sample.iter_mut() { *out = value; }
                    }
                },
                cpal::UnknownTypeBuffer::I16(mut buffer) => {
                    for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                        let value = (value * std::i16::MAX as f32) as i16;
                        for out in sample.iter_mut() { *out = value; }
                    }
                },
                cpal::UnknownTypeBuffer::F32(mut buffer) => {
                    for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(&mut data_source) {
                        for out in sample.iter_mut() { *out = value; }
                    }
                },
                */
                cpal::UnknownTypeBuffer::U16(mut buffer) => {
                    debug!("U16");
                    panic!();
                },
                cpal::UnknownTypeBuffer::I16(mut buffer) => {
                    debug!("I16");
                    for (sample, value) in buffer.chunks_mut(format.channels.len()).zip(data_source.iter()) {
                        for out in sample.iter_mut() { *out = *value; }
                    }
                },
                cpal::UnknownTypeBuffer::F32(mut buffer) => {
                    debug!("F32");
                    panic!();
                },
            };

            Ok(())
        })).execute(executor);
        debug!("Spawned executor task");
        */

        /*
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(500));
                voice.pause();
                thread::sleep(Duration::from_millis(500));
                voice.play();
            }
        });
        */

        thread::spawn(move || {
            event_loop.run();
        });

        Player { voice: voice }
    }
}

struct SamplesIterator<T> {
    rx: Receiver<Vec<T>>,
    buf: VecDeque<T>,
}

impl<T> SamplesIterator<T> {
    fn new(rx: Receiver<Vec<T>>) -> SamplesIterator<T> {
        SamplesIterator {
            rx: rx,
            buf: VecDeque::new(),
        }
    }
}

impl<T> Iterator for SamplesIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.buf.len() == 0 {
            match self.rx.try_recv() {
                Ok(r) => {
                    let mut vec = r.into_iter().collect();
                    self.buf.append(&mut vec);
                    self.buf.pop_front()
                },
                Err(_) => None
            }
        } else {
            self.buf.pop_front()
        }
    }
}

pub struct Audio {
    endpoint: cpal::Endpoint,
    format: cpal::Format,
    voice: cpal::Voice,
    tx: Sender<Vec<i16>>,
}

impl Audio {
    pub fn new() -> Audio {
        let allowed_sample_rates = vec![cpal::SamplesRate(48000), cpal::SamplesRate(44100), cpal::SamplesRate(96000)];
        let endpoint = cpal::get_default_endpoint().unwrap();

        struct Match {
            sr: usize,
            chan: usize,
            data_type: cpal::SampleFormat,
            format: cpal::Format,
        }

        let mut best_match = None;
        let formats = endpoint.get_supported_formats_list().expect("No audio formats found");
        for f in formats {
            let s = allowed_sample_rates.iter().position(|x| *x == f.samples_rate);
            let c = f.channels.len();
            let d = f.data_type;
            if s.is_none() { continue; }
            let s = s.unwrap();
            let new_m = Match {sr: s, chan: c, data_type: d, format: f.clone() };
            if best_match.is_none() {
                best_match = Some(new_m);
                continue;
            }
            if best_match.as_ref().unwrap().sr > s {
                best_match = Some(new_m);
                continue;
            }
            if best_match.as_ref().unwrap().sr < s {
                continue;
            }
            if best_match.as_ref().unwrap().chan > c {
                best_match = Some(new_m);
                continue;
            }
            if best_match.as_ref().unwrap().chan < c {
                continue;
            }
            if d == cpal::SampleFormat::I16 {
                best_match = Some(new_m);
            }
        }

        let best_match = best_match.expect("No supported audio format found");
        let format = best_match.format;
        let channels = format.channels.len();

        let executor = Arc::new(AudioExecutor);
        let event_loop = cpal::EventLoop::new();

        let (mut voice, stream) = cpal::Voice::new(&endpoint, &format, &event_loop).unwrap();

        let (tx, rx) = channel();
        let mut samples = SamplesIterator::new(rx);

        task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
            match buffer {
                cpal::UnknownTypeBuffer::U16(mut buffer) => {
                    for (sample, value) in buffer.chunks_mut(channels).
                        zip(&mut samples) {
                        let value = ((value as i32) + ::std::i16::MAX as i32) as u16;
                        for out in sample.iter_mut() { *out = value; }
                    }
                },

                cpal::UnknownTypeBuffer::I16(mut buffer) => {
                    for (sample, value) in buffer.chunks_mut(channels).
                        zip(&mut samples) {
                        for out in sample.iter_mut() { *out = value; }
                    }
                },

                cpal::UnknownTypeBuffer::F32(mut buffer) => {
                    for (sample, value) in buffer.chunks_mut(channels).
                        zip(&mut samples) {
                        let value = (value as f32) / ::std::i16::MAX as f32;
                        for out in sample.iter_mut() { *out = value; }
                    }
                },
            }

            Ok(())
        })).execute(executor);

        voice.play();

        thread::spawn(move || { event_loop.run() });

        Audio {
            endpoint: endpoint,
            format: format,
            voice: voice,
            tx: tx,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        let cpal::SamplesRate(rate) = self.format.samples_rate;
        rate
    }

    pub fn add_samples(&mut self, samples: Vec<i16>) {
        let _ = self.tx.send(samples).unwrap();
    }
}

struct ChildContainer(Child);

impl Write for ChildContainer {
    fn write(&mut self, buffer: &[u8]) -> IoResult<usize> {
        self.0.stdin.as_mut().unwrap().write(buffer)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

struct PcmSink<W: Write + Send + 'static>(bool, W);

/*
impl<W: Write + Send> AudioSource for PcmSink<W> {
    fn is_stereo(&mut self) -> bool {
        self.0
    }

    fn write_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
        /*
        for (i, v) in buffer.iter_mut().enumerate() {
            *v = match self.1.read_i16::<LittleEndian>() {
                Ok(v) => v,
                Err(ref e) => return if e.kind() == IoErrorKind::UnexpectedEof {
                    Some(i)
                } else {
                    None
                },
            }
        }

        Some(buffer.len())
        */
        Some(0)
    }
}
*/

fn create_ffmpeg_command<P: AsRef<OsStr>>(path: P) -> SerenityResult<Child> {
    let path = path.as_ref();

    /*
    /// Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path).unwrap_or(false);
    */
    let is_stereo = true;
    let stereo_val = if is_stereo {
        "2"
    } else {
        "1"
    };

    /*
    let args = [
        "-f",
        "s16le",
        "-ac",
        stereo_val,
        "-ar",
        "48000",
        "-i",
        "-",
        "-f",
        "s16le",
        "-acodec",
        "pcm_s16le",
        "-ar",
        "16000",
    ];
    */
    /*
    let args = [
        "-f",
        "s16le",
        "-ac",
        stereo_val,
        "-ar",
        "48000",
        "-re",
        "-i",
        "-",
    ];
    */
    let args = [
        "-f",
        "s16le",
        "-ar",
        "48000",
        "-i",
        "-",
        "-acodec",
        "copy",
    ];

    let command = Command::new("ffmpeg")
        .args(&args)
        .arg(path)
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn ffmpeg command");
    debug!("Spawned ffmpeg command");

    Ok(command)
}

pub fn ffmpeg<P: AsRef<OsStr>>(path: P, data: &[i16]) -> SerenityResult<()> {
    let mut command = create_ffmpeg_command(path)?;

    // TODO: replace this with a safe solution.
    let mut buffer: &[u8] = unsafe { ::std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2) };
    {
        let mut ffmpeg_stdin = command.stdin.as_mut().unwrap();
        for v in data.iter() {
            ffmpeg_stdin.write_i16::<LittleEndian>(*v);
        }
    }

    drop(command.stdin.take());
    //io::copy(&mut buffer, &mut ffmpeg_stdin).expect("Failed to write data to ffmpeg pipe");
    debug!("Wrote data to ffmpeg pipe");

    {
        let mut ffmpeg_stderr = command.stderr.as_mut().unwrap();
        let mut ffmpeg_stdout = command.stdout.as_mut().unwrap();
        let mut stderr = String::new();
        let mut stdout = String::new();
        ffmpeg_stderr.read_to_string(&mut stderr).unwrap();
        ffmpeg_stdout.read_to_string(&mut stdout).unwrap();

        debug!("stderr: {}", stderr);
        debug!("stdout: {}", stdout);
    }
    drop(command.stderr.take());
    drop(command.stdout.take());

    Ok(())
}

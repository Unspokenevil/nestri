use crate::enc_helper::Codec::{Audio, Video};
use crate::enc_helper::{AudioCodec, Codec, EncoderType, VideoCodec};
use clap::ValueEnum;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RateControlCQP {
    /// Constant Quantization Parameter (CQP) quality level
    pub quality: u32,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RateControlVBR {
    /// Target bitrate in kbps
    pub target_bitrate: u32,
    /// Maximum bitrate in kbps
    pub max_bitrate: u32,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RateControlCBR {
    /// Target bitrate in kbps
    pub target_bitrate: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, ValueEnum)]
pub enum RateControlMethod {
    CQP,
    VBR,
    CBR,
}
impl RateControlMethod {
    pub fn as_str(&self) -> &str {
        match self {
            RateControlMethod::CQP => "cqp",
            RateControlMethod::VBR => "vbr",
            RateControlMethod::CBR => "cbr",
        }
    }
}
impl FromStr for RateControlMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cqp" => Ok(RateControlMethod::CQP),
            "vbr" => Ok(RateControlMethod::VBR),
            "cbr" => Ok(RateControlMethod::CBR),
            _ => Err(format!("Invalid rate control method: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RateControl {
    /// Constant Quantization Parameter
    CQP(RateControlCQP),
    /// Variable Bitrate
    VBR(RateControlVBR),
    /// Constant Bitrate
    CBR(RateControlCBR),
}

#[derive(Debug, PartialEq, Eq, Clone, ValueEnum)]
pub enum LatencyControl {
    LowestLatency,
    HighestQuality,
}

pub struct EncodingOptionsBase {
    /// Codec (e.g. "h264", "opus" etc.)
    pub codec: Codec,
    /// Overridable encoder (e.g. "vah264lpenc", "opusenc" etc.)
    pub encoder: Option<String>,
    /// Rate control method (e.g. "cqp", "vbr", "cbr")
    pub rate_control: RateControl,
    /// Latency control option, what to tweak settings towards (latency or quality)
    pub latency_control: LatencyControl,
}
impl EncodingOptionsBase {
    pub fn debug_print(&self) {
        tracing::info!("> Codec: '{}'", self.codec.as_str());
        tracing::info!("> Encoder: '{}'", self.encoder.as_deref().unwrap_or("auto"));
        match &self.rate_control {
            RateControl::CQP(cqp) => {
                tracing::info!("> Rate Control: CQP");
                tracing::info!("-> Quality: {}", cqp.quality);
            }
            RateControl::VBR(vbr) => {
                tracing::info!("> Rate Control: VBR");
                tracing::info!("-> Target Bitrate: {}", vbr.target_bitrate);
                tracing::info!("-> Max Bitrate: {}", vbr.max_bitrate);
            }
            RateControl::CBR(cbr) => {
                tracing::info!("> Rate Control: CBR");
                tracing::info!("-> Target Bitrate: {}", cbr.target_bitrate);
            }
        }
        match &self.latency_control {
            LatencyControl::LowestLatency => {
                tracing::info!("> Latency Control: Priorizing lowest latency");
            }
            LatencyControl::HighestQuality => {
                tracing::info!("> Latency Control: Priorizing quality at the cost of latency");
            }
        }
    }
}

pub struct VideoEncodingOptions {
    pub base: EncodingOptionsBase,
    pub encoder_type: EncoderType,
    pub bit_depth: u32,
    pub keyframe_dist_secs: u32,
}
impl VideoEncodingOptions {
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            base: EncodingOptionsBase {
                codec: Video(
                    matches
                        .get_one::<VideoCodec>("video-codec")
                        .unwrap_or(&VideoCodec::H264)
                        .clone(),
                ),
                encoder: matches.get_one::<String>("video-encoder").cloned(),
                rate_control: match matches
                    .get_one::<RateControlMethod>("video-rate-control")
                    .unwrap_or(&RateControlMethod::CBR)
                {
                    RateControlMethod::CQP => RateControl::CQP(RateControlCQP {
                        quality: matches
                            .get_one::<String>("video-cqp")
                            .unwrap()
                            .parse::<u32>()
                            .unwrap(),
                    }),
                    RateControlMethod::CBR => RateControl::CBR(RateControlCBR {
                        target_bitrate: matches.get_one::<u32>("video-bitrate").unwrap().clone(),
                    }),
                    RateControlMethod::VBR => RateControl::VBR(RateControlVBR {
                        target_bitrate: matches.get_one::<u32>("video-bitrate").unwrap().clone(),
                        max_bitrate: matches.get_one::<u32>("video-bitrate-max").unwrap().clone(),
                    }),
                },
                latency_control: matches
                    .get_one::<LatencyControl>("video-latency-control")
                    .unwrap_or(&LatencyControl::LowestLatency)
                    .clone(),
            },
            encoder_type: matches
                .get_one::<EncoderType>("video-encoder-type")
                .unwrap_or(&EncoderType::HARDWARE)
                .clone(),
            bit_depth: matches
                .get_one::<u32>("video-bit-depth")
                .copied()
                .unwrap_or(8),
            keyframe_dist_secs: matches
                .get_one::<u32>("keyframe-dist-secs")
                .copied()
                .unwrap_or(1),
        }
    }

    pub fn debug_print(&self) {
        tracing::info!("Video Encoding Options:");
        self.base.debug_print();
        tracing::info!("> Encoder Type: {}", self.encoder_type.as_str());
        tracing::info!("> Bit Depth: {}", self.bit_depth);
        tracing::info!("> Keyframe Distance Seconds: {}", self.keyframe_dist_secs);
    }
}
impl Deref for VideoEncodingOptions {
    type Target = EncodingOptionsBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Debug, PartialEq, Eq, Clone, ValueEnum)]
pub enum AudioCaptureMethod {
    PULSEAUDIO,
    PIPEWIRE,
    ALSA,
}
impl AudioCaptureMethod {
    pub fn as_str(&self) -> &str {
        match self {
            AudioCaptureMethod::PULSEAUDIO => "PulseAudio",
            AudioCaptureMethod::PIPEWIRE => "PipeWire",
            AudioCaptureMethod::ALSA => "ALSA",
        }
    }
}
impl FromStr for AudioCaptureMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pulseaudio" => Ok(AudioCaptureMethod::PULSEAUDIO),
            "pipewire" => Ok(AudioCaptureMethod::PIPEWIRE),
            "alsa" => Ok(AudioCaptureMethod::ALSA),
            _ => Err(format!("Invalid audio capture method: {}", s)),
        }
    }
}

pub struct AudioEncodingOptions {
    pub base: EncodingOptionsBase,
    pub capture_method: AudioCaptureMethod,
}
impl AudioEncodingOptions {
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            base: EncodingOptionsBase {
                codec: Audio(
                    matches
                        .get_one::<AudioCodec>("audio-codec")
                        .unwrap_or(&AudioCodec::OPUS)
                        .clone(),
                ),
                encoder: matches.get_one::<String>("audio-encoder").cloned(),
                rate_control: match matches
                    .get_one::<RateControlMethod>("audio-rate-control")
                    .unwrap_or(&RateControlMethod::CBR)
                {
                    RateControlMethod::CBR => RateControl::CBR(RateControlCBR {
                        target_bitrate: matches.get_one::<u32>("audio-bitrate").unwrap().clone(),
                    }),
                    RateControlMethod::VBR => RateControl::VBR(RateControlVBR {
                        target_bitrate: matches.get_one::<u32>("audio-bitrate").unwrap().clone(),
                        max_bitrate: matches.get_one::<u32>("audio-bitrate-max").unwrap().clone(),
                    }),
                    wot => panic!("Invalid rate control method for audio: {}", wot.as_str()),
                },
                latency_control: matches
                    .get_one::<LatencyControl>("audio-latency-control")
                    .unwrap_or(&LatencyControl::LowestLatency)
                    .clone(),
            },
            capture_method: matches
                .get_one::<AudioCaptureMethod>("audio-capture-method")
                .unwrap_or(&AudioCaptureMethod::PIPEWIRE)
                .clone(),
        }
    }

    pub fn debug_print(&self) {
        tracing::info!("Audio Encoding Options:");
        self.base.debug_print();
        tracing::info!("> Capture Method: {}", self.capture_method.as_str());
    }
}
impl Deref for AudioEncodingOptions {
    type Target = EncodingOptionsBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

pub struct EncodingArgs {
    /// Video encoder options
    pub video: VideoEncodingOptions,
    /// Audio encoder options
    pub audio: AudioEncodingOptions,
}
impl EncodingArgs {
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            video: VideoEncodingOptions::from_matches(matches),
            audio: AudioEncodingOptions::from_matches(matches),
        }
    }

    pub fn debug_print(&self) {
        tracing::info!("Encoding Arguments:");
        self.video.debug_print();
        self.audio.debug_print();
    }
}

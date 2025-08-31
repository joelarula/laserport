use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::error::Error;
use std::thread;
use std::time::Duration;

// Define enums for channel options based on the device manual

#[derive(Clone, Copy)]
pub enum MainSwitch {
    Off,  // 0-9
    On,   // 10-255
}

impl MainSwitch {
    fn to_u8(&self) -> u8 {
        match self {
            MainSwitch::Off => 0,
            MainSwitch::On => 255,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ColorMode {
    FixedWhite,      // 0-9 (assuming ranges, adjust as per fixed colors)
    FixedRed,        // 10-19 etc., but manual has 0-69 for fixed, need sub-divide if needed
    // ... add more fixed
    OverallChange,   // 70-79
    PatternInitial,  // 80-89
    Rainbow,         // 90-92
    Seg2,            // 93-110
    Seg3,            // 111-131
    Seg4,            // 132-149
    Seg8,            // 150-182
    Seg16,           // 183-218
    Seg32,           // 219-253
    Gradient,        // 254-255
}

impl ColorMode {
    fn to_u8(&self) -> u8 {
        match self {
            ColorMode::FixedWhite => 0,
            ColorMode::FixedRed => 10, // Approximate, refine based on exact fixed color breaks
            // Add more...
            ColorMode::OverallChange => 75,
            ColorMode::PatternInitial => 85,
            ColorMode::Rainbow => 91,
            ColorMode::Seg2 => 100,
            ColorMode::Seg3 => 120,
            ColorMode::Seg4 => 140,
            ColorMode::Seg8 => 160,
            ColorMode::Seg16 => 200,
            ColorMode::Seg32 => 230,
            ColorMode::Gradient => 255,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ColorFlow {
    NoChange,             // 0-9
    Forward(u8),          // 10-127: slow to fast
    Reverse(u8),          // 128-255: slow to fast
}

impl ColorFlow {
    fn to_u8(&self) -> u8 {
        match self {
            ColorFlow::NoChange => 0,
            ColorFlow::Forward(speed) => 10 + speed.clamp(&0, &117),
            ColorFlow::Reverse(speed) => 128 + speed.clamp(&0, &127),
        }
    }
}

#[derive(Clone, Copy)]
pub enum GraphicsGroup {
    Static1,  // 0-24: basic geo
    Static2,  // 25-49
    Static3,  // 50-74: edge highlight
    Static4,  // 75-99: dot/punched
    Static5,  // 100-124: Christmas
    Animation1, // 125-149
    Animation2, // 150-174
    Animation3, // 175-199
    Animation4, // 200-224
    Animation5, // 225-255
}

impl GraphicsGroup {
    fn to_u8(&self) -> u8 {
        match self {
            GraphicsGroup::Static1 => 10,
            GraphicsGroup::Static2 => 35,
            GraphicsGroup::Static3 => 60,
            GraphicsGroup::Static4 => 85,
            GraphicsGroup::Static5 => 110,
            GraphicsGroup::Animation1 => 135,
            GraphicsGroup::Animation2 => 160,
            GraphicsGroup::Animation3 => 185,
            GraphicsGroup::Animation4 => 210,
            GraphicsGroup::Animation5 => 235,
        }
    }
}

// CH5: Pattern selection 0-255, raw u8

#[derive(Clone, Copy)]
pub enum DynamicEffect {
    None,                 // 0-1
    Single(u8),           // 2-206: one per 2 values, so u8 1-102
    LineRandom,           // 207-216
    AnimationRandom,      // 217-226
    ChristmasRandom,      // 227-236
    OutdoorRandom,        // 237-246
    AllRandom,            // 247-255
}

impl DynamicEffect {
    fn to_u8(&self) -> u8 {
        match self {
            DynamicEffect::None => 0,
            DynamicEffect::Single(id) => 2 + (id.clamp(&0, &102) * 2),
            DynamicEffect::LineRandom => 210,
            DynamicEffect::AnimationRandom => 220,
            DynamicEffect::ChristmasRandom => 230,
            DynamicEffect::OutdoorRandom => 240,
            DynamicEffect::AllRandom => 250,
        }
    }
}

// CH7: Effect speed 0-1 default, 2-255 slow-fast, raw u8 or enum Default / Manual(u8)

// CH8: Pattern size 0-255 manual

#[derive(Clone, Copy)]
pub enum AutoScaling {
    SizeOption(u8),       // 0-15
    SmallToLarge(u8),     // 16-55: speed sel
    LargeToSmall(u8),     // 56-95
    ScalingSpeed(u8),     // 96-135
    TwoPointIrregular,    // 136-175
    ThreeQuarterIrregular,// 176-215
    QuadraticIrregular,   // 216-255
}

impl AutoScaling {
    fn to_u8(&self) -> u8 {
        match self {
            AutoScaling::SizeOption(val) => (*val).clamp(0, 15),
            AutoScaling::SmallToLarge(speed) => 16 + (*speed).clamp(0, 39),
            AutoScaling::LargeToSmall(speed) => 56 + (*speed).clamp(0, 39),
            AutoScaling::ScalingSpeed(speed) => 96 + (*speed).clamp(0, 39),
            AutoScaling::TwoPointIrregular => 150,
            AutoScaling::ThreeQuarterIrregular => 190,
            AutoScaling::QuadraticIrregular => 230,
        }
    }
}

#[derive(Clone, Copy)]
pub enum RotationCenter {
    Angle(u8),        // 0-127
    ForwardSpeed(u8), // 128-191
    ReverseSpeed(u8), // 192-255
}

impl RotationCenter {
    fn to_u8(&self) -> u8 {
        match self {
            RotationCenter::Angle(angle) => (*angle).clamp(0, 127),
            RotationCenter::ForwardSpeed(speed) => 128 + (*speed).clamp(0, 63),
            RotationCenter::ReverseSpeed(speed) => 192 + (*speed).clamp(0, 63),
        }
    }
}

// Similar for CH11: Horizontal flip (around X-axis? manual says rotates around X-axis, but desc flip)
#[derive(Clone, Copy)]
pub enum FlipHorizontal {
    Position(u8),  // 0-127
    Speed(u8),     // 128-255
}

impl FlipHorizontal {
    fn to_u8(&self) -> u8 {
        match self {
            FlipHorizontal::Position(pos) => (*pos).clamp(0, 127),
            FlipHorizontal::Speed(speed) => 128 + (*speed).clamp(0, 127),
        }
    }
}

// CH12: Vertical flip, same as above

// CH13: Horizontal movement
#[derive(Clone, Copy)]
pub enum MovementHorizontal {
    Position(u8),         // 0-127
    CircularSpeed(u8),    // 128-255
}

impl MovementHorizontal {
    fn to_u8(&self) -> u8 {
        match self {
            MovementHorizontal::Position(pos) => (*pos).clamp(0, 127),
            MovementHorizontal::CircularSpeed(speed) => 128 + (*speed).clamp(0, 127),
        }
    }
}

// CH14: Vertical movement, same

#[derive(Clone, Copy)]
pub enum WavesX {
    None,  // 0-1
    AmpSpeed(u8),  // 2-255: 8 gears, every 32
}

impl WavesX {
    fn to_u8(&self) -> u8 {
        match self {
            WavesX::None => 0,
            WavesX::AmpSpeed(gear) => 2 + (gear.clamp(&0, &7) * 32),
        }
    }
}

#[derive(Clone, Copy)]
pub enum GradualDrawing {
    None,                 // 0-1
    Manual1,              // 2-63
    Manual2,              // 64-127
    AutoClockwise(u8),    // 128-153: slow-fast
    AutoCounter(u8),      // 154-179
    AutoIncDecReverse,    // 180-205
    AutoIncDecSame,       // 206-255
}

impl GradualDrawing {
    fn to_u8(&self) -> u8 {
        match self {
            GradualDrawing::None => 0,
            GradualDrawing::Manual1 => 30,
            GradualDrawing::Manual2 => 90,
            GradualDrawing::AutoClockwise(speed) => 128 + speed.clamp(&0, &25),
            GradualDrawing::AutoCounter(speed) => 154 + speed.clamp(&0, &25),
            GradualDrawing::AutoIncDecReverse => 190,
            GradualDrawing::AutoIncDecSame => 220,
        }
    }
}

// Struct for the laser device state (16 channels)
pub struct LaserState {
    pub ch1: MainSwitch,
    pub ch2: ColorMode,
    pub ch3: ColorFlow,
    pub ch4: GraphicsGroup,
    pub ch5: u8,  // Pattern select 0-255
    pub ch6: DynamicEffect,
    pub ch7: u8,  // Speed 0-255
    pub ch8: u8,  // Size 0-255
    pub ch9: AutoScaling,
    pub ch10: RotationCenter,
    pub ch11: FlipHorizontal,  // Around X
    pub ch12: FlipHorizontal,  // Around Y, same type
    pub ch13: MovementHorizontal,
    pub ch14: MovementHorizontal,  // Vertical, same type
    pub ch15: WavesX,
    pub ch16: GradualDrawing,
}

impl LaserState {
    pub fn new() -> Self {
        LaserState {
            ch1: MainSwitch::Off,
            ch2: ColorMode::FixedWhite,
            ch3: ColorFlow::NoChange,
            ch4: GraphicsGroup::Static1,
            ch5: 0,
            ch6: DynamicEffect::None,
            ch7: 0,
            ch8: 128,  // Mid size
            ch9: AutoScaling::SizeOption(0),
            ch10: RotationCenter::Angle(0),
            ch11: FlipHorizontal::Position(0),
            ch12: FlipHorizontal::Position(0),
            ch13: MovementHorizontal::Position(64),  // Center-ish
            ch14: MovementHorizontal::Position(64),
            ch15: WavesX::None,
            ch16: GradualDrawing::None,
        }
    }

    pub fn to_channels(&self) -> [u8; 16] {
        [
            self.ch1.to_u8(),
            self.ch2.to_u8(),
            self.ch3.to_u8(),
            self.ch4.to_u8(),
            self.ch5,
            self.ch6.to_u8(),
            self.ch7,
            self.ch8,
            self.ch9.to_u8(),
            self.ch10.to_u8(),
            self.ch11.to_u8(),
            self.ch12.to_u8(),
            self.ch13.to_u8(),
            self.ch14.to_u8(),
            self.ch15.to_u8(),
            self.ch16.to_u8(),
        ]
    }
}

// DMX Controller for serial port
pub struct DmxController {
    port: Box<dyn serialport::SerialPort>,
    address: usize,  // Starting channel (1-based)
}

impl DmxController {
    pub fn new(port_name: &str, address: usize) -> Result<Self, Box<dyn Error>> {
        let mut port = serialport::new(port_name, 250_000)
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::Two)
            .timeout(Duration::from_millis(10))
            .open()?;

        Ok(DmxController { port, address: address - 1 })  // 0-based index
    }

    pub fn send(&mut self, state: &LaserState) -> Result<(), Box<dyn Error>> {
        self.port.set_break()?;
        thread::sleep(Duration::from_micros(100));
        self.port.clear_break()?;

        let mut frame: Vec<u8> = vec![0x00];  // Start code
        frame.resize(513, 0);  // 1 + 512

        let channels = state.to_channels();
        for (i, &val) in channels.iter().enumerate() {
            let idx = self.address + i + 1;  // +1 for after start
            if idx < frame.len() {
                frame[idx] = val;
            }
        }

        self.port.write_all(&frame)?;
        Ok(())
    }
}

// Example usage
fn main() -> Result<(), Box<dyn Error>> {
    let mut controller = DmxController::new("COM4", 1)?;

    let mut state = LaserState::new();
    state.ch1 = MainSwitch::On;
    state.ch2 = ColorMode::OverallChange;
    state.ch3 = ColorFlow::Forward(50);

    controller.send(&state)?;

    Ok(())
}
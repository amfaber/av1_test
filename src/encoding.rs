use glam::Vec2;
use rav1e::{prelude::{SpeedSettings, ChromaSampling}, Config, EncoderConfig, EncoderStatus};

use crate::frame_generation::{FrameIterator, RGB};

pub fn test_encoding() {
    let width = 200;
    let height = 200;
    let enc = EncoderConfig {
        width,
        height,
        speed_settings: SpeedSettings::from_preset(9),
        chroma_sampling: ChromaSampling::Cs444,
        ..Default::default()
    };
    let cfg = Config::new().with_encoder_config(enc.clone());

    let mut ctx = cfg.new_context::<u8>().unwrap();

    let mut frame_iterator = FrameIterator{
        width,
        height,
        position: Vec2::splat(0.5),
        velocity: Vec2::new(0.3, 0.4),
        circle_color: RGB{ r: 10., b: 0., g: 0. },
        background: RGB{ r: 0., g: 0., b: 0., },
        radius: 0.05,
        delta_t: 0.03,
    };

    let n_frames = 100;
    for _ in 0..n_frames {
        let mut frame = ctx.new_frame();
        let generated = frame_iterator.next().unwrap();
        let y = generated.iter().map(|yuv| (yuv.y * 255.) as u8).collect::<Vec<_>>();
        let u = generated.iter().map(|yuv| (yuv.u * 255.) as u8).collect::<Vec<_>>();
        let v = generated.iter().map(|yuv| (yuv.v * 255.) as u8).collect::<Vec<_>>();

        frame.planes[0].copy_from_raw_u8(&y, width, 1);
        frame.planes[1].copy_from_raw_u8(&u, width, 1);
        frame.planes[2].copy_from_raw_u8(&v, width, 1);
        ctx.send_frame(frame).unwrap();
    }
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("test.ivf")
        .unwrap();

    ivf::write_ivf_header(&mut output_file, width, height, 30, 1);

    ctx.flush();
    loop {
        match ctx.receive_packet() {
            Ok(packet) => {
                ivf::write_ivf_frame(&mut output_file, packet.input_frameno * 1, &packet.data);
                // dbg!(packet.input_frameno);
            }

            Err(EncoderStatus::LimitReached) => break,

            Err(e) => {
                // dbg!(e);
            }
        }
    }
}

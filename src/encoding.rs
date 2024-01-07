use glam::Vec2;
use rav1e::{
    prelude::*,
    Config, EncoderConfig, EncoderStatus,
};

use crate::frame_generation::{FrameIterator, RGB};

pub fn test_encoding() {
    let width = 200;
    let height = 200;
    let enc = EncoderConfig {
        width,
        height,
        speed_settings: SpeedSettings::from_preset(9),
        chroma_sampling: ChromaSampling::Cs444,
        color_description: Some(ColorDescription{
            color_primaries: ColorPrimaries::BT709,
            transfer_characteristics: TransferCharacteristics::BT709,
            matrix_coefficients: MatrixCoefficients::BT709,
        }),
        ..Default::default()
    };
    let cfg = Config::new().with_encoder_config(enc.clone());

    let mut ctx = cfg.new_context::<u8>().unwrap();

    let mut frame_iterator = FrameIterator {
        width,
        height,
        position: Vec2::splat(0.5),
        velocity: Vec2::new(0.3, 0.4),
        circle_color: RGB {
            r: 1.,
            b: 0.,
            g: 0.,
        },
        background: RGB {
            r: 0.,
            g: 0.,
            b: 0.,
        },
        radius: 0.05,
        delta_t: 0.03,
    };

    let n_frames = 100;
    for _ in 0..n_frames {
        let mut frame = ctx.new_frame();
        let generated = frame_iterator.next().unwrap();
        let y = generated
            .iter()
            .map(|ycbcr| (ycbcr.y * 255.) as u8)
            .collect::<Vec<_>>();
        let u = generated
            .iter()
            // .map(|ycbcr| (ycbcr.cb * 255.) as u8)
            .map(|ycbcr| ((ycbcr.cb + 0.5) * 255.) as u8)
            .collect::<Vec<_>>();
        let v = generated
            .iter()
            // .map(|ycbcr| (ycbcr.cr * 255.) as u8)
            .map(|ycbcr| ((ycbcr.cr + 0.5) * 255.) as u8)
            .collect::<Vec<_>>();

        frame.planes[0].copy_from_raw_u8(&y, width, 1);
        frame.planes[1].copy_from_raw_u8(&u, width, 1);
        frame.planes[2].copy_from_raw_u8(&v, width, 1);
        ctx.send_frame(frame).unwrap();
    }
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("test.mp4")
        .unwrap();
    ctx.flush();

    let mut writer = mp4::Mp4Writer::write_start(
        &mut output_file,
        &mp4::Mp4Config {
            major_brand: "isom".parse().unwrap(),
            minor_version: 1,
            compatible_brands: vec![
                "av01".parse().unwrap(),
                "isom".parse().unwrap(),
                "mp42".parse().unwrap(),
            ],
            timescale: 1000,
        },
    )
    .unwrap();
    writer
        .add_track(&mp4::TrackConfig {
            track_type: mp4::TrackType::Video,
            timescale: 30,
            language: "English".to_string(),
            media_conf: mp4::MediaConfig::Av1Config(mp4::Av1Config {
                width: width as _,
                height: height as _,
                sequence_header: ctx.container_sequence_header(),
            }),
        })
        .unwrap();

    let mut i = 0;
    loop {
        match ctx.receive_packet() {
            Ok(packet) => {
                i += 1;
                writer
                    .write_sample(
                        1,
                        &mp4::Mp4Sample {
                            start_time: packet.input_frameno,
                            duration: 1,
                            rendering_offset: 0,
                            is_sync: matches!(packet.frame_type, FrameType::KEY),
                            bytes: packet.data.into(),
                        },
                    )
                    .unwrap();
            }

            Err(EncoderStatus::LimitReached) => break,

            Err(e) => {
                // panic!("{}", e)
            }
        }
    }
    dbg!(i);
    writer.write_end().unwrap();
}

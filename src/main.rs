use std::{fmt::Write, fs::File, io::stdout, time::Duration};

use anyhow::{Context, Result};
use crossterm::{
    cursor, execute,
    style::{Color, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::{thread_rng, Rng};

const FPS: u64 = 20;
const VELOCITY: (f32, f32) = (0.05, 0.05);

fn get_random_colour() -> Color {
    use Color::*;

    let mut rng = thread_rng();
    match rng.gen_range(0..7) {
        0 => Red,
        1 => Green,
        2 => Yellow,
        3 => Blue,
        4 => Magenta,
        5 => Cyan,
        6 => White,
        _ => unreachable!(),
    }
}

// pos: (col, row)
fn draw_image(bytes: &[Vec<&u8>], position: (usize, usize)) -> Result<()> {
    let mut buf = format!(
        "{}",
        cursor::MoveTo(position.0 as u16, position.1 as u16 / 2,)
    );

    let mut bytes = bytes.to_owned();
    if position.1 % 2 != 0 {
        bytes.insert(0, vec![&0; bytes[0].len()])
    }

    for (y, row) in bytes.iter().step_by(2).enumerate() {
        for (x, byte) in row.iter().enumerate() {
            write!(
                &mut buf,
                "{}",
                match (
                    byte,
                    bytes.get(2 * y + 1).unwrap_or(&vec![&0; bytes[0].len()])[x]
                ) {
                    (0, 0) => " ",
                    (0xff, 0) => "\u{2580}",
                    (0, 0xff) => "\u{2584}",
                    (0xff, 0xff) => "\u{2588}",
                    _ => unreachable!(),
                }
            )
            .context("failed to write to stdout buffer")?;
        }

        write!(
            &mut buf,
            "{}{}",
            cursor::MoveToColumn(position.0 as u16),
            cursor::MoveDown(1)
        )
        .context("failed to write to stdout buffer")?;
    }
    print!("{}", buf);

    {
        use std::io::Write;
        let _ = stdout().flush();
    }

    Ok(())
}

fn main() -> Result<()> {
    let decoder = png::Decoder::new(File::open("dvd.png").context("failed to open `dvd.png`")?);
    let mut reader = decoder
        .read_info()
        .context("failed to read info from image decoder")?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .context("failed to read frame from PNG")?;
    let image_width = info.width;
    let image_height = info.height;

    let bytes = &buf[..info.buffer_size()];

    let image =
        bytes
            .iter()
            .skip(3)
            .step_by(4)
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, byte)| {
                if i % image_width as usize == 0 {
                    acc.push(vec![byte]);
                    acc
                } else {
                    acc.last_mut().unwrap().push(byte);
                    acc
                }
            });

    execute!(stdout(), terminal::EnterAlternateScreen, cursor::Hide)
        .context("failed to enter alternate screen or hide cursor")?;

    let (terminal_width, terminal_height) =
        terminal::size().context("failed to query terminal dimensions")?;

    let mut velocity = VELOCITY;
    let mut position = (0.0, 0.0);

    // mainloop
    loop {
        print!(
            "{}{}",
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );
        draw_image(&image, (position.0 as usize, position.1 as usize))?;

        if position.0 as u32 + image_width > terminal_width as u32 || position.0 < 0.0 {
            velocity.0 *= -1.0;
            print!("{}", SetForegroundColor(get_random_colour()));
        }
        if position.1 as u32 + image_height > terminal_height as u32 * 2 || position.1 < 0.0 {
            velocity.1 *= -1.0;
            print!("{}", SetForegroundColor(get_random_colour()));
        }

        position.0 += velocity.0 / FPS as f32;
        position.1 += velocity.1 / FPS as f32;

        std::thread::sleep(Duration::from_secs(1 / FPS));
    }
}

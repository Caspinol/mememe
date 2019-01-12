use image::{GenericImageView, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, FontCollection, Scale};

struct Point {
  pub x: u32,
  pub y: u32,
}

pub struct Meme<'a> {
  text: &'a str,
  position: Point,
  scale: Scale
}

impl<'a> Meme<'a> {
  pub fn new<S>(text: S, x: u32, y: u32, s: u32) -> Self
  where
    S: Into<&'a str>,
  {
    Self {
      text: text.into(),
      position: Point { x: x + 2, y: y + 2 },
      scale: Scale{ x: (s as f32) * 1.7, y: s as f32 }
    }
  }

  pub fn render_text(&self, infile: &str, outfile: &str) {
    let white: Rgba<u8> = Rgba([255u8, 255u8, 255u8, 255u8]);
    let black: Rgba<u8> = Rgba([0u8, 0u8, 0u8, 255u8]);
    let mut image = image::open(infile).unwrap();
    let font = Vec::from(include_bytes!("../impact.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font)
      .unwrap()
      .into_font()
      .unwrap();

    let img_width = image.width();

    let wrapped_text = wrap_text(self.text, img_width, &font, &self.scale);

    let outline_tickness = 2;

    let mut vert_offset = self.position.y;
    for text in wrapped_text {
      draw_text_mut(
        &mut image,
        black,
        self.position.x - outline_tickness,
        vert_offset,
        self.scale,
        &font,
        &text,
      );
      draw_text_mut(
        &mut image,
        black,
        self.position.x + outline_tickness,
        vert_offset,
        self.scale,
        &font,
        &text,
      );
      draw_text_mut(
        &mut image,
        black,
        self.position.x,
        vert_offset - outline_tickness,
        self.scale,
        &font,
        &text,
      );
      draw_text_mut(
        &mut image,
        black,
        self.position.x,
        vert_offset + outline_tickness,
        self.scale,
        &font,
        &text,
      );
      draw_text_mut(
        &mut image,
        white,
        self.position.x,
        vert_offset,
        self.scale,
        &font,
        &text,
      );
      vert_offset += self.scale.y as u32;
    }

    image.save(outfile).unwrap();
  }
}

fn get_text_width(text: &str, font: &Font, scale: &Scale) -> u32 {
  5 + font
    .glyphs_for(text.chars())
    .map(|g| g.scaled(*scale).h_metrics().advance_width)
    .sum::<f32>() as u32
}

fn wrap_text(text: &str, width: u32, font: &Font, scale: &Scale) -> Vec<String> {
  let mut wrapped_text = Vec::new();

  let mut line = String::new();
  let mut line_remaining = width;
  for word in text.split(' ') {
    let word_width = get_text_width(word, font, scale);
    if word_width < line_remaining {
      line.push_str(&format!("{} ", word));
      line_remaining = line_remaining.checked_sub(word_width + 5).unwrap_or(0);
      continue;
    } else {
      wrapped_text.push(line);
      line = String::new();
      line.push_str(&format!("{} ", word));
      line_remaining = width.checked_sub(word_width + 5).unwrap_or(0);
    }
  }
  line.pop(); // remove the trailing space
  wrapped_text.push(line);
  wrapped_text
}

#[cfg(test)]
mod tests {

  use super::wrap_text;
  use super::{FontCollection, Scale};
  #[test]
  fn test_word_wrap() {
    let font = Vec::from(include_bytes!("../impact.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font)
      .unwrap()
      .into_font()
      .unwrap();

    let height = 12.4;
    let scale = Scale {
      x: height * 2.0,
      y: height,
    };

    let text = "Hello World!";

    let wrapped = wrap_text(text, 500, &font, &scale);
    assert_eq!(wrapped.len(), 1);
    assert_eq!(wrapped[0], "Hello World!".to_owned());

    let text = "Some very very long text that will most likely need some wrapping to fit it into the required width";
    let wrapped = wrap_text(text, 500, &font, &scale);
    assert!(wrapped.len() == 3);

    let text = "";
    let wrapped = wrap_text(text, 500, &font, &scale);
    assert!(wrapped.len() == 1);
  }
}

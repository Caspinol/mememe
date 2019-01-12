#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate image;
extern crate imageproc;
extern crate multipart;
extern crate rand;
extern crate rocket_cors;
extern crate rusttype;

mod random;
mod render;

use rocket::data::{self, FromDataSimple};
use rocket::http::{Method};
use rocket::response::NamedFile;
use rocket::{Data, Outcome::*, Request};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use std::io::{BufWriter, Cursor, Read, Write};

use std::fs::File;
use std::path::{Path};

const NAME_LIMIT: u64 = 1024;

#[derive(Debug)]
struct MemeForm {
  posx: u32,
  posy: u32,
  scale: u32,
  caption: String,
  base_image: Vec<u8>,
  img_name: String,
}

impl FromDataSimple for MemeForm {
  type Error = String;

  fn from_data(request: &Request, data: Data) -> data::Outcome<Self, Self::Error> {
    let ct = request
      .headers()
      .get_one("Content-Type")
      .expect("no content-type");
    let idx = ct.find("boundary=").expect("no boundary");
    let boundary = &ct[(idx + "boundary=".len())..];

    let mut d = Vec::new();
    data.stream_to(&mut d).expect("Unable to read");

    use multipart::server::Multipart;
    let mut mp = Multipart::with_body(Cursor::new(d), boundary);

    let mut posx = 0;
    let mut posy = 0;
    let mut scale = 0;
    let mut caption = String::new();
    let mut base_image = Vec::new();
    let mut img_name = String::new();

    if let Err(e) = mp.foreach_entry(|mut entry| match (*entry.headers.name).as_ref() {
      "posx" => {
        let mut buf = String::new();
        let _ = entry.data.read_to_string(&mut buf);
        posx = buf.parse::<u32>().unwrap_or(0);
      }
      "posy" => {
        let mut buf = String::new();
        let _ = entry.data.read_to_string(&mut buf);
        posy = buf.parse::<u32>().unwrap_or(0);
      }
      "caption" => {
        let _ = entry.data.read_to_string(&mut caption);
      }
      "scale" => {
        let mut buf = String::new();
        let _ = entry.data.read_to_string(&mut buf);
        scale = buf.parse::<u32>().unwrap_or(0);
      }
      "file" => {
        let _ = entry.data.read_to_end(&mut base_image);
      }
      "img_name" => {
        let _ = entry.data.read_to_string(&mut img_name);
      }
      _ => {}
    }) {
      println!("Error: {}", e)
    }

    Success(MemeForm {
      posx,
      posy,
      scale,
      caption,
      base_image,
      img_name,
    })
  }
}

#[post("/meme", data = "<meme_details>")]
fn meme(meme_details: MemeForm) -> Result<Vec<u8>, std::io::Error> {
  use self::random::random_name;

  let meme_file = format!("static/{}{}", random_name(30), meme_details.img_name);

  let file = File::create(Path::new(&meme_file))?;
  let mut writer = BufWriter::new(file);
  writer
    .write_all(meme_details.base_image.as_slice())
    .expect("Failed to write to file");

  use self::render::Meme;
  let meme = Meme::new(
    meme_details.caption.as_str(),
    meme_details.posx,
    meme_details.posy,
    meme_details.scale
  );
  meme.render_text(&meme_file, &meme_file);
  let mut file = NamedFile::open(meme_file)?;

  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer)?;
  Ok(buffer)
}

fn main() {
  let (allowed_origins, _failed_origins) = AllowedOrigins::some(&["http://localhost:8080"]);
  let options = rocket_cors::Cors {
    allowed_origins: allowed_origins,
    allowed_methods: vec![Method::Post]
      .into_iter()
      .map(From::from)
      .collect(),
    allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
    allow_credentials: true,
    ..Default::default()
  };

  rocket::ignite()
    .mount("/", routes![meme])
    .attach(options)
    .launch();
}

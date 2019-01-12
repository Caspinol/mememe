# Meme server

> When the 1000 words is still not enough.

Easlily add captions to the image

## Instalation

```bash
git clone https://github.com/Caspinol/mememe
cd mememe
cargo run
```

## Usage

```bash
curl -XPOST -F'posx=3' -F'file=@soon_to_be_meme_image.jpg' -F'posy=2' -F'scale=80' -F'caption=Hello meme' -F'img_name=soon_to_be_meme_image.jpg' 'http://localhost:8000/meme' > the_ultimate_meme.jpg
```

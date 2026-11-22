///|
pub impl @definition.Text for Image with fn parse(text) {
  image(text)
}

///|
pub impl @definition.Text for Image with fn print(value) {
  image_text(value.register, value.memory)
}

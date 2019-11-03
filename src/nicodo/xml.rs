use super::{error, Comment, Result};
use quick_xml::{
  events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
  Writer,
};
use std::io::Write;

pub fn write_json<W: Write>(writer: W, comments: &[Comment]) -> Result<()> {
  serde_json::to_writer(writer, comments).map_err(|_| error::Error::Serialization)?;
  Ok(())
}

pub fn write_xml<W: Write>(writer: W, comments: &[Comment]) -> Result<()> {
  // let mut w = Writer::new(writer);
  let mut w = Writer::new_with_indent(writer, ' ' as u8, 0);

  for e in &[
    Event::Decl(BytesDecl::new(b"1.0", Some(b"utf-8"), None)),
    Event::Start(BytesStart::owned(b"packet".to_vec(), "packet".len())),
    {
      let mut e = BytesStart::owned(b"thread".to_vec(), "thread".len());
      e.push_attribute(("last_res", &(comments.len() - 1).to_string() as &str));
      e.push_attribute(("ticket", ""));
      Event::Empty(e)
    },
    {
      let mut e = BytesStart::owned(b"view_counter".to_vec(), "view_counter".len());
      e.push_attribute(("video", "0"));
      Event::Empty(e)
    },
  ] {
    w.write_event(e).map_err(|_| error::Error::Serialization)?;
  }

  for c in comments.iter() {
    if c.content.is_empty() {
      continue;
    }

    w.write_event({
      let mut e = BytesStart::owned(b"chat".to_vec(), "chat".len());
      e.push_attribute(("date", &c.date.to_string() as &str));
      e.push_attribute(("vpos", &c.vpos.to_string() as &str));
      e.push_attribute(("no", &c.no.to_string() as &str));
      if let Some(user_id) = c.user_id.as_ref() {
        e.push_attribute(("user_id", &user_id.to_string() as &str));
      }
      if let Some(mail) = c.mail.as_ref() {
        e.push_attribute(("mail", &mail.to_string() as &str));
      }
      Event::Start(e)
    })
    .and_then(|_| {
      w.write_event(Event::Text(BytesText::from_plain_str(
        &c.content.to_string(),
      )))
    })
    .and_then(|_| w.write_event(Event::End(BytesEnd::owned(b"chat".to_vec()))))
    .map_err(|_| error::Error::Serialization)?;
  }

  w.write_event(Event::End(BytesEnd::borrowed(b"packet")))
    .map_err(|_| error::Error::Serialization)?;

  Ok(())
}
